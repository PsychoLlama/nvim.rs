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
#![allow(private_interfaces)] // FFI structs: C callers use raw pointers only

use std::ffi::{c_char, c_int, c_void};

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

/// Read and apply the SN_COMPOUND section from a buffer to slang_T.
/// Replaces C read_compound(). Parses header bytes, comppat patterns,
/// compound flags, and calls vim_regcomp for sl_compprog.
///
/// Returns 0 on success, SP_*ERROR on failure.
///
/// # Safety
/// - `buf` must be valid for `len` bytes.
/// - `slang` must be a valid non-null pointer to a SlangRaw.
/// - `vim_regcomp` may call back into C code.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_read_compound(
    buf: *const u8,
    len: usize,
    slang: *mut crate::SlangRaw,
) -> c_int {
    const MAXWLEN_VAL: c_int = 254;
    // RE_MAGIC + RE_STRING + RE_STRICT
    const RE_FLAGS: c_int = 1 + 4 + 16;

    if len < 2 {
        return SP_FORMERROR;
    }
    let buf_slice = std::slice::from_raw_parts(buf, len);
    let mut offset = 0usize;

    if len < 3 {
        return SP_FORMERROR;
    }

    // <compmax>
    let c = buf_slice[offset] as c_int;
    offset += 1;
    (*slang).sl_compmax = if c < 2 { MAXWLEN_VAL } else { c };

    // <compminlen>
    let c = buf_slice[offset] as c_int;
    offset += 1;
    (*slang).sl_compminlen = if c < 1 { 0 } else { c };

    // <compsylmax>
    let c = buf_slice[offset] as c_int;
    offset += 1;
    (*slang).sl_compsylmax = if c < 1 { MAXWLEN_VAL } else { c };

    // Check for compoptions (Vim 7.0b+ format)
    if offset < len && buf_slice[offset] != 0 {
        // Old format: treat remaining bytes as flags directly
    } else if offset + 1 < len && buf_slice[offset] == 0 {
        offset += 1; // skip 0 byte
        (*slang).sl_compoptions = buf_slice[offset] as c_int;
        offset += 1;

        if offset + 2 > len {
            return SP_TRUNCERROR;
        }
        let pat_count = ((buf_slice[offset] as c_int) << 8) | (buf_slice[offset + 1] as c_int);
        offset += 2;

        // Initialize sl_comppat garray
        let gap = std::ptr::addr_of_mut!((*slang).sl_comppat);
        (*gap).ga_len = 0;
        (*gap).ga_maxlen = 0;
        (*gap).ga_itemsize = std::mem::size_of::<*mut c_char>() as i32;
        (*gap).ga_growsize = pat_count.max(1);
        (*gap).ga_data = std::ptr::null_mut();
        ga_grow(gap, pat_count);

        // Read <comppatlen> <comppattext> pairs
        for _ in 0..pat_count {
            if offset >= len {
                return SP_TRUNCERROR;
            }
            let pat_len = buf_slice[offset] as usize;
            offset += 1;
            if offset + pat_len > len {
                return SP_TRUNCERROR;
            }
            let pat_str = xmalloc_spell(pat_len + 1).cast::<c_char>();
            std::ptr::copy_nonoverlapping(buf.add(offset).cast::<c_char>(), pat_str, pat_len);
            *pat_str.add(pat_len) = 0;
            let slot = ((*gap).ga_data as *mut *mut c_char).add((*gap).ga_len as usize);
            *slot = pat_str;
            (*gap).ga_len += 1;
            offset += pat_len;
        }
    } else {
        return SP_FORMERROR;
    }

    if offset > len {
        return SP_FORMERROR;
    }

    // Parse compound flags with Rust
    let flags_len = len - offset;
    if flags_len == 0 {
        return SP_FORMERROR;
    }

    let flags_slice = std::slice::from_raw_parts(buf.add(offset), flags_len);
    let flags_result = match parse_compound_flags(flags_slice) {
        Ok(r) => r,
        Err(e) => return e,
    };

    // Copy startflags
    let sf_len = flags_result.startflags_len as usize;
    let sf_ptr = xmalloc_spell(sf_len + 1).cast::<u8>();
    std::ptr::copy_nonoverlapping(flags_result.startflags.as_ptr(), sf_ptr, sf_len + 1);
    (*slang).sl_compstartflags = sf_ptr;

    // Copy allflags
    let af_len = flags_result.allflags_len as usize;
    let af_ptr = xmalloc_spell(af_len + 1).cast::<u8>();
    std::ptr::copy_nonoverlapping(flags_result.allflags.as_ptr(), af_ptr, af_len + 1);
    (*slang).sl_compallflags = af_ptr;

    // Copy comprules if valid
    if flags_result.comprules_valid {
        let cr_len = flags_result.comprules_len as usize;
        let cr_ptr = xmalloc_spell(cr_len + 1).cast::<u8>();
        std::ptr::copy_nonoverlapping(flags_result.comprules.as_ptr(), cr_ptr, cr_len + 1);
        (*slang).sl_comprules = cr_ptr;
    } else {
        (*slang).sl_comprules = std::ptr::null_mut();
    }

    // Compile the regex via vim_regcomp (must be called from C/Rust via FFI)
    let pat_len = flags_result.pattern_len as usize;
    let prog = vim_regcomp(flags_result.pattern.as_ptr().cast::<c_char>(), RE_FLAGS);
    if prog.is_null() {
        return SP_FORMERROR;
    }
    (*slang).sl_compprog = prog;

    // Suppress unused import warning
    let _ = pat_len;

    0 // OK
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

/// Read and apply the SN_SAL section from a buffer to slang_T.
/// Replaces C read_sal_section(). Parses header, allocates salitem_T array,
/// fills wide-char strings, and calls rs_set_sal_first.
///
/// # Safety
/// - `buf` must be valid for `len` bytes.
/// - `slang` must be a valid non-null pointer to a SlangRaw.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_read_sal_section(
    buf: *const u8,
    len: usize,
    slang: *mut crate::SlangRaw,
) -> c_int {
    const SAL_F0LLOWUP: u8 = 1;
    const SAL_COLLAPSE: u8 = 2;
    const SAL_REM_ACCENTS: u8 = 4;

    (*slang).sl_sofo = false;

    if len < 3 {
        return SP_FORMERROR;
    }
    let buf_slice = std::slice::from_raw_parts(buf, len);

    // Parse SAL header: <salflags> <salcount>
    let Some((sal_header, mut offset)) = parse_sal_header(buf_slice) else {
        return SP_FORMERROR;
    };

    // Apply flags
    if sal_header.flags & SAL_F0LLOWUP != 0 {
        (*slang).sl_followup = true;
    }
    if sal_header.flags & SAL_COLLAPSE != 0 {
        (*slang).sl_collapse = true;
    }
    if sal_header.flags & SAL_REM_ACCENTS != 0 {
        (*slang).sl_rem_accents = true;
    }

    let cnt = sal_header.count as usize;

    // Initialize sl_sal garray
    let gap = std::ptr::addr_of_mut!((*slang).sl_sal);
    (*gap).ga_len = 0;
    (*gap).ga_maxlen = 0;
    (*gap).ga_itemsize = std::mem::size_of::<crate::SalitemT>() as i32;
    (*gap).ga_growsize = 10;
    (*gap).ga_data = std::ptr::null_mut();
    ga_grow(gap, (cnt + 1) as c_int);

    // Parse each SAL item
    for _ in 0..cnt {
        let item_res = parse_sal_item(&buf_slice[offset..]);
        let (rs_item, item_consumed) = match item_res {
            Ok(r) => r,
            Err(e) => return e,
        };
        offset += item_consumed;

        let smp = ((*gap).ga_data as *mut crate::SalitemT).add((*gap).ga_len as usize);

        // Allocate from buffer (lead+NUL+oneof+NUL+rules+NUL)
        let from_used = rs_item.from_used as usize;
        let p = xmalloc_spell(from_used + 1).cast::<c_char>();
        std::ptr::copy_nonoverlapping(rs_item.from.as_ptr().cast::<c_char>(), p, from_used);

        (*smp).sm_lead = p.add(rs_item.lead_offset as usize);
        (*smp).sm_leadlen = rs_item.lead_len as c_int;

        if rs_item.oneof_offset == 0xFFFF {
            (*smp).sm_oneof = std::ptr::null_mut();
        } else {
            (*smp).sm_oneof = p.add(rs_item.oneof_offset as usize);
        }
        (*smp).sm_rules = p.add(rs_item.rules_offset as usize);

        if rs_item.has_to && rs_item.to_len > 0 {
            let to_len = rs_item.to_len as usize;
            let to_p = xmalloc_spell(to_len + 1).cast::<c_char>();
            std::ptr::copy_nonoverlapping(rs_item.to.as_ptr().cast::<c_char>(), to_p, to_len);
            *to_p.add(to_len) = 0;
            (*smp).sm_to = to_p;
        } else {
            (*smp).sm_to = std::ptr::null_mut();
        }

        // Build wide-char versions
        (*smp).sm_lead_w = rs_mb_str2wide((*smp).sm_lead);
        (*smp).sm_leadlen = mb_charlen((*smp).sm_lead);
        (*smp).sm_oneof_w = if (*smp).sm_oneof.is_null() {
            std::ptr::null_mut()
        } else {
            rs_mb_str2wide((*smp).sm_oneof)
        };
        (*smp).sm_to_w = if (*smp).sm_to.is_null() {
            std::ptr::null_mut()
        } else {
            rs_mb_str2wide((*smp).sm_to)
        };

        (*gap).ga_len += 1;
    }

    // Add sentinel entry with empty sm_lead
    if (*gap).ga_len > 0 {
        let smp = ((*gap).ga_data as *mut crate::SalitemT).add((*gap).ga_len as usize);
        let p = xmalloc_spell(1).cast::<c_char>();
        *p = 0;
        (*smp).sm_lead = p;
        (*smp).sm_lead_w = rs_mb_str2wide(p);
        (*smp).sm_leadlen = 0;
        (*smp).sm_oneof = std::ptr::null_mut();
        (*smp).sm_oneof_w = std::ptr::null_mut();
        (*smp).sm_rules = p;
        (*smp).sm_to = std::ptr::null_mut();
        (*smp).sm_to_w = std::ptr::null_mut();
        (*gap).ga_len += 1;
    }

    // Fill the first-index table
    rs_set_sal_first(slang);

    0 // OK
}

/// Read and apply the SN_PREFCOND section from a FILE* stream to slang_T.
///
/// Replaces C read_prefcond_section(). Reads prefix condition count, then
/// for each condition reads length+bytes, builds "^pattern", calls vim_regcomp.
///
/// Returns 0 on success, SP_*ERROR on failure.
///
/// # Safety
/// - `fd` must be a valid FILE* pointer.
/// - `slang` must be a valid non-null pointer to a SlangRaw.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_read_prefcond_section(
    fd: *mut libc::FILE,
    slang: *mut crate::SlangRaw,
) -> c_int {
    const MAXWLEN: usize = 254;
    // RE_MAGIC | RE_STRING
    const RE_FLAGS: c_int = 1 + 4;

    // Read 2-byte count header
    let mut cnt_buf = [0u8; 2];
    if libc::fread(cnt_buf.as_mut_ptr().cast(), 1, 2, fd) != 2 {
        return if libc::feof(fd) != 0 {
            SP_TRUNCERROR
        } else {
            SP_OTHERERROR
        };
    }
    let cnt = ((cnt_buf[0] as usize) << 8) | (cnt_buf[1] as usize);
    if cnt == 0 {
        return SP_FORMERROR;
    }

    // Allocate sl_prefprog array (zeroed = NULL regprog pointers)
    let prog_arr = libc::calloc(cnt, std::mem::size_of::<*mut std::ffi::c_void>())
        .cast::<*mut std::ffi::c_void>();
    if prog_arr.is_null() {
        return SP_OTHERERROR;
    }
    (*slang).sl_prefprog = prog_arr;
    (*slang).sl_prefixcnt = cnt as i32;

    for i in 0..cnt {
        // Read 1-byte condition length
        let n = libc::fgetc(fd);
        if n < 0 {
            return SP_TRUNCERROR;
        }
        let n = n as usize;
        if n >= MAXWLEN {
            return SP_FORMERROR;
        }
        if n == 0 {
            continue; // empty condition
        }

        // Read condition bytes (non-NUL)
        let mut cond_buf = [0u8; MAXWLEN];
        if libc::fread(cond_buf.as_mut_ptr().cast(), 1, n, fd) != n {
            return if libc::feof(fd) != 0 {
                SP_TRUNCERROR
            } else {
                SP_OTHERERROR
            };
        }
        // Check for NUL
        if cond_buf[..n].contains(&0) {
            return SP_FORMERROR;
        }

        // Build "^pattern\0"
        let mut pattern = [0u8; MAXWLEN + 2];
        pattern[0] = b'^';
        pattern[1..=n].copy_from_slice(&cond_buf[..n]);
        pattern[n + 1] = 0;

        let prog = vim_regcomp(pattern.as_ptr().cast::<c_char>(), RE_FLAGS);
        *prog_arr.add(i) = prog;
    }
    0 // OK
}

/// C fromto_T struct layout: two heap-allocated char* strings.
#[repr(C)]
struct FromtoC {
    ft_from: *mut c_char,
    ft_to: *mut c_char,
}

/// Read and apply a REP or REPSAL section from FILE* to a garray_T and first[] table.
/// Replaces C read_rep_section(). Reads count, items (from/to strings), fills first table.
///
/// # Safety
/// - `fd` must be a valid FILE* pointer.
/// - `gap` must be a valid non-null pointer to a GArrayRaw (garray_T for fromto_T).
/// - `first` must be a valid pointer to an array of at least 256 i16 values.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_read_rep_section(
    fd: *mut libc::FILE,
    gap: *mut crate::GArrayRaw,
    first: *mut i16,
) -> c_int {
    // Read 2-byte count header
    let mut cnt_buf = [0u8; 2];
    if libc::fread(cnt_buf.as_mut_ptr().cast(), 1, 2, fd) != 2 {
        return if libc::feof(fd) != 0 {
            SP_TRUNCERROR
        } else {
            SP_OTHERERROR
        };
    }
    let cnt = ((cnt_buf[0] as usize) << 8) | (cnt_buf[1] as usize);

    ga_grow(gap, cnt as c_int);

    while (*gap).ga_len < cnt as c_int {
        let ftp = ((*gap).ga_data as *mut FromtoC).add((*gap).ga_len as usize);

        // Read from_len
        let from_len = libc::fgetc(fd);
        if from_len < 0 {
            return SP_TRUNCERROR;
        }
        if from_len == 0 {
            return SP_FORMERROR;
        }
        let from_len = from_len as usize;

        // Read from string
        let mut from_buf = [0u8; 256];
        if libc::fread(from_buf.as_mut_ptr().cast(), 1, from_len, fd) != from_len {
            return if libc::feof(fd) != 0 {
                SP_TRUNCERROR
            } else {
                SP_OTHERERROR
            };
        }

        // Read to_len
        let to_len = libc::fgetc(fd);
        if to_len < 0 {
            return SP_TRUNCERROR;
        }
        let to_len = to_len as usize;

        // Read to string
        let mut to_buf = [0u8; 256];
        if to_len > 0 && libc::fread(to_buf.as_mut_ptr().cast(), 1, to_len, fd) != to_len {
            return if libc::feof(fd) != 0 {
                SP_TRUNCERROR
            } else {
                SP_OTHERERROR
            };
        }

        // Allocate NUL-terminated copies (xmemdupz pattern)
        let from_ptr = xmalloc_spell(from_len + 1).cast::<c_char>();
        std::ptr::copy_nonoverlapping(from_buf.as_ptr().cast::<c_char>(), from_ptr, from_len);
        *from_ptr.add(from_len) = 0;
        (*ftp).ft_from = from_ptr;

        let to_ptr = xmalloc_spell(to_len + 1).cast::<c_char>();
        std::ptr::copy_nonoverlapping(to_buf.as_ptr().cast::<c_char>(), to_ptr, to_len);
        *to_ptr.add(to_len) = 0;
        (*ftp).ft_to = to_ptr;

        (*gap).ga_len += 1;
    }

    // Fill first-index table
    std::ptr::write_bytes(first, 0xFF, 256); // initialize to -1 (0xFFFF for i16)
    for i in 0..(*gap).ga_len as usize {
        let ftp = ((*gap).ga_data as *const FromtoC).add(i);
        let first_byte = *(*ftp).ft_from as u8 as usize;
        if *first.add(first_byte) == -1i16 {
            *first.add(first_byte) = i as i16;
        }
    }
    0 // OK
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
// Prefix Condition Section Writing
// =============================================================================

/// Write the prefix condition section to a buffer.
///
/// Format: <prefcondcnt (2 bytes BE)>
///         for each condition: <condlen (1 byte)> <condstr (N bytes)>
///
/// `strings` is a slice of byte slices (each is a NUL-terminated C string
/// or a raw byte slice). Returns the number of bytes written.
pub fn write_prefcond_section(buf: &mut [u8], strings: &[&[u8]]) -> Option<usize> {
    let count = strings.len();
    if count > 0xFFFF {
        return None;
    }

    // Calculate total size needed.
    let mut total = 2; // <prefcondcnt>
    for s in strings {
        total += 1 + s.len(); // <condlen> + <condstr>
    }

    if buf.len() < total {
        return None;
    }

    // Write <prefcondcnt>
    let count_bytes = (count as u16).to_be_bytes();
    buf[0] = count_bytes[0];
    buf[1] = count_bytes[1];
    let mut offset = 2;

    // Write each <condlen> <condstr>
    for s in strings {
        let len = s.len();
        if len > 255 {
            return None;
        }
        buf[offset] = len as u8;
        offset += 1;
        buf[offset..offset + len].copy_from_slice(s);
        offset += len;
    }

    Some(offset)
}

/// FFI wrapper for writing the prefix condition section.
///
/// `strs` is an array of `count` pointers to NUL-terminated C strings.
/// Each string is written as `<condlen (1 byte)> <condstr>`.
///
/// # Safety
/// All pointers must be valid. `strs` must point to `count` valid C string pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_write_prefcond_section(
    buf: *mut u8,
    buf_len: usize,
    strs: *const *const u8,
    count: usize,
    written_out: *mut usize,
) -> c_int {
    if buf.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }
    if count > 0 && strs.is_null() {
        return SP_OTHERERROR;
    }

    // Collect string slices from C string pointers.
    let mut string_slices: Vec<&[u8]> = Vec::with_capacity(count);
    for i in 0..count {
        let ptr = *strs.add(i);
        if ptr.is_null() {
            // NULL pointer means empty string.
            string_slices.push(&[]);
        } else {
            // Find the NUL terminator.
            let mut len = 0;
            while *ptr.add(len) != 0 {
                len += 1;
            }
            string_slices.push(std::slice::from_raw_parts(ptr, len));
        }
    }

    let out_slice = std::slice::from_raw_parts_mut(buf, buf_len);
    match write_prefcond_section(out_slice, &string_slices) {
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
// Dictionary / Wordfile / Add-word Parsing (Phase 4)
// =============================================================================

/// Result of parsing one line from a .dic dictionary file.
///
/// The `word` slice holds the word bytes (without trailing NUL).
/// `word_len` is the length of the word.
/// `affix_offset` is the byte offset within the original `line` buffer where
/// the affix list starts (i.e. past the `/` separator), or `u16::MAX` when
/// there is no affix list.
/// `affix_len` is the length of the affix string in the original buffer.
///
/// Note: the escaped content is written back into a caller-supplied
/// `word_buf` so that the C side never needs to allocate for parsing.
#[repr(C)]
pub struct DicLineResult {
    /// Byte offset in the caller-supplied `word_buf` where word starts (always 0).
    pub word_len: u16,
    /// Byte offset in the original `line` buffer where the affix list starts,
    /// or `0xFFFF` when absent.
    pub affix_offset: u16,
    /// Length of the affix string in the original `line` buffer.
    pub affix_len: u16,
}

/// Parse one line from a .dic dictionary file.
///
/// The function:
/// 1. Trims trailing CR/LF/whitespace.
/// 2. Skips comment lines that start with `#` or `/`.
/// 3. Processes escape sequences `\\` → `\` and `\/` → `/`.
/// 4. Splits the word from its affix list at the first unescaped `/`.
///
/// On success returns `0` and fills `result_out` and `word_buf`.
/// Returns `1` when the line is empty or a comment (caller should skip).
/// Returns `-1` on error (NULL pointers, buffer too small).
///
/// # Safety
/// All pointer parameters must be valid for their indicated lengths.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_dic_line(
    line: *const u8,
    line_len: usize,
    word_buf: *mut u8,
    word_buf_cap: usize,
    result_out: *mut DicLineResult,
) -> c_int {
    if line.is_null() || word_buf.is_null() || result_out.is_null() {
        return -1;
    }

    let src = std::slice::from_raw_parts(line, line_len);

    // Trim trailing CR/LF/whitespace
    let mut end = src.len();
    while end > 0 && src[end - 1] <= b' ' {
        end -= 1;
    }
    let src = &src[..end];

    // Empty line or comment
    if src.is_empty() || src[0] == b'#' || src[0] == b'/' {
        return 1;
    }

    // Process escape sequences and find affix separator.
    // Write unescaped word into word_buf.
    let dst = std::slice::from_raw_parts_mut(word_buf, word_buf_cap);
    let mut wi = 0usize; // write index into dst
    let mut ri = 0usize; // read index into src
    let mut affix_offset: u16 = 0xFFFF;
    let mut affix_len: u16 = 0;

    while ri < src.len() {
        let c = src[ri];
        if c == b'\\' && ri + 1 < src.len() && (src[ri + 1] == b'\\' || src[ri + 1] == b'/') {
            // Escape sequence: consume two bytes, emit one
            if wi >= word_buf_cap {
                return -1;
            }
            dst[wi] = src[ri + 1];
            wi += 1;
            ri += 2;
        } else if c == b'/' {
            // Affix separator: record position and stop writing word
            affix_offset = (ri + 1) as u16;
            affix_len = (src.len().saturating_sub(ri + 1)) as u16;
            break;
        } else {
            if wi >= word_buf_cap {
                return -1;
            }
            dst[wi] = c;
            wi += 1;
            ri += 1;
        }
    }

    *result_out = DicLineResult {
        word_len: wi as u16,
        affix_offset,
        affix_len,
    };
    0
}

// ---- Wordfile line parsing --------------------------------------------------

/// Flags returned from `rs_parse_wordfile_line`.
/// These values must match the C-side `WF_*` constants.
pub mod wordfile_flags {
    /// Word with `!` flag: banned / bad word.
    pub const WF_BANNED: i32 = 0x10;
    /// Word with `?` flag: rare word.
    pub const WF_RARE: i32 = 0x08;
    /// Word with `=` flag: keep-case word (implies FIXCAP).
    pub const WF_KEEPCAP: i32 = 0x80;
    /// keep-case word, ALLCAP not allowed (set together with KEEPCAP).
    pub const WF_FIXCAP: i32 = 0x40;
    /// Word has a region restriction.
    pub const WF_REGION: i32 = 0x01;
}

/// Result of parsing one line from a plain wordfile (.add / raw wordlist).
#[repr(C)]
pub struct WordfileLineResult {
    /// NUL-terminated kind tag for directive lines.
    /// `b"encoding\0"` or `b"regions\0"` when the line is a directive.
    /// Empty (first byte NUL) for ordinary word lines.
    pub directive: [u8; 16],
    /// For directive lines: length of the value that follows in `directive_value`.
    /// For word lines: length of the word (word starts at byte 0 of line buffer).
    pub word_len: u16,
    /// Byte offset in the original `line` buffer where the word ends (before `/flags`).
    pub word_end_offset: u16,
    /// Accumulated `WF_*` word flags (0 for directive lines).
    pub flags: i32,
    /// Accumulated region bitmask (0 = no region restriction parsed).
    /// Caller must combine with `spin->si_region` when `WF_REGION` not set.
    pub regionmask: i32,
    /// For `/regions=` directive: the region count (number of 2-char pairs).
    pub region_count: u8,
}

/// Parse one line from a plain wordfile (.add or raw word list).
///
/// Returns:
/// - `0`: ordinary word line; `result_out.word_len` and `result_out.flags` are filled.
/// - `1`: directive line (`/encoding=` or `/regions=`); `result_out.directive` is filled.
/// - `2`: line should be skipped (comment, empty, unknown `/` directive).
/// - `-1`: error (NULL pointer).
///
/// The function does **not** handle encoding conversion (stays in C); it only
/// classifies and splits the line.
///
/// # Safety
/// All pointer parameters must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_wordfile_line(
    line: *const u8,
    line_len: usize,
    region_count: i32,
    result_out: *mut WordfileLineResult,
) -> c_int {
    if line.is_null() || result_out.is_null() {
        return -1;
    }

    let src = std::slice::from_raw_parts(line, line_len);

    // Trim trailing CR/LF/whitespace
    let mut end = src.len();
    while end > 0 && src[end - 1] <= b' ' {
        end -= 1;
    }
    let src = &src[..end];

    // Empty line
    if src.is_empty() {
        return 2;
    }

    // Comment line
    if src[0] == b'#' {
        return 2;
    }

    // Directive line: starts with '/'
    if src[0] == b'/' {
        let body = &src[1..];

        let mut res = WordfileLineResult {
            directive: [0u8; 16],
            word_len: 0,
            word_end_offset: 0,
            flags: 0,
            regionmask: 0,
            region_count: 0,
        };

        if body.starts_with(b"encoding=") {
            let tag = b"encoding";
            res.directive[..tag.len()].copy_from_slice(tag);
            // Value starts at byte 9 in body (after "encoding=")
            res.word_len = (body.len() - 9) as u16;
            res.word_end_offset = 10; // 1 (slash) + 9 ("encoding=")
            *result_out = res;
            return 1;
        }

        if body.starts_with(b"regions=") {
            let tag = b"regions";
            res.directive[..tag.len()].copy_from_slice(tag);
            let value = &body[8..]; // after "regions="
            res.word_len = value.len() as u16;
            res.word_end_offset = 9; // 1 (slash) + 8 ("regions=")
            res.region_count = (value.len() / 2) as u8;
            *result_out = res;
            return 1;
        }

        // Unknown directive: skip
        return 2;
    }

    // Ordinary word line: find optional `/flags` suffix
    let slash_pos = src.iter().position(|&b| b == b'/');
    let word_end = slash_pos.unwrap_or(src.len());

    let mut flags: i32 = 0;
    let mut regionmask: i32 = 0;

    if let Some(slash) = slash_pos {
        let flag_bytes = &src[slash + 1..];
        let mut i = 0;
        while i < flag_bytes.len() {
            let b = flag_bytes[i];
            if b == b'=' {
                flags |= wordfile_flags::WF_KEEPCAP | wordfile_flags::WF_FIXCAP;
            } else if b == b'!' {
                flags |= wordfile_flags::WF_BANNED;
            } else if b == b'?' {
                flags |= wordfile_flags::WF_RARE;
            } else if b.is_ascii_digit() {
                let digit = (b - b'0') as i32;
                if digit == 0 || digit > region_count {
                    // Invalid region: caller will emit error message.
                    // Return code 3 so C can log and break.
                    return 3;
                }
                if (flags & wordfile_flags::WF_REGION) == 0 {
                    regionmask = 0; // first region digit clears default
                }
                flags |= wordfile_flags::WF_REGION;
                regionmask |= 1 << (digit - 1);
            } else {
                // Unrecognized flag: caller will emit warning and break.
                return 4;
            }
            i += 1;
        }
    }

    *result_out = WordfileLineResult {
        directive: [0u8; 16],
        word_len: word_end as u16,
        word_end_offset: word_end as u16,
        flags,
        regionmask,
        region_count: 0,
    };
    0
}

// ---- spell_add_word helpers -------------------------------------------------

/// Format the line to append to a `.add` spell file for `zg`/`zw`.
///
/// - `what == 0` (`SPELL_ADD_GOOD`): `"word\n"`
/// - `what == 1` (`SPELL_ADD_BAD`):  `"word/!\n"`
/// - `what == 2` (`SPELL_ADD_RARE`): `"word/?\n"`
///
/// Returns the number of bytes written into `buf_out`, or `-1` on error.
///
/// # Safety
/// All pointer parameters must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_add_word_format(
    word: *const u8,
    word_len: usize,
    what: i32,
    buf_out: *mut u8,
    buf_cap: usize,
) -> c_int {
    if word.is_null() || buf_out.is_null() {
        return -1;
    }

    let w = std::slice::from_raw_parts(word, word_len);
    let dst = std::slice::from_raw_parts_mut(buf_out, buf_cap);

    // Determine suffix: "" / "/!" / "/?"
    let suffix: &[u8] = match what {
        1 => b"/!",
        2 => b"/?",
        _ => b"",
    };

    let total = word_len + suffix.len() + 1; // +1 for '\n'
    if total > buf_cap {
        return -1;
    }

    dst[..word_len].copy_from_slice(w);
    let mut pos = word_len;
    dst[pos..pos + suffix.len()].copy_from_slice(suffix);
    pos += suffix.len();
    dst[pos] = b'\n';

    c_int::try_from(total).unwrap_or(-1)
}

/// Search for a duplicate word in the content of a `.add` file.
///
/// The function scans `file_content` line by line looking for a line that
/// starts with `word` (of length `word_len`) followed by either `/` or a
/// byte `< ' '` (end of line), matching the C logic in `spell_add_word`.
///
/// On match, writes the byte offset of the *start of that line* into
/// `*offset_out` and returns `true`.  Returns `false` when no match found.
///
/// # Safety
/// All pointer parameters must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_find_duplicate_word(
    file_content: *const u8,
    content_len: usize,
    word: *const u8,
    word_len: usize,
    offset_out: *mut usize,
) -> bool {
    if file_content.is_null() || word.is_null() || offset_out.is_null() {
        return false;
    }

    let content = std::slice::from_raw_parts(file_content, content_len);
    let w = std::slice::from_raw_parts(word, word_len);

    let mut pos = 0usize;
    while pos < content.len() {
        let line_start = pos;

        // Find end of line
        let mut line_end = pos;
        while line_end < content.len() && content[line_end] != b'\n' {
            line_end += 1;
        }

        let line = &content[pos..line_end];

        // Check if this line starts with the word followed by '/' or EOL
        if line.len() >= word_len && &line[..word_len] == w {
            let after = if word_len < line.len() {
                line[word_len]
            } else {
                b'\n' // virtual newline at EOL
            };
            if after == b'/' || after < b' ' {
                *offset_out = line_start;
                return true;
            }
        }

        pos = line_end + 1; // skip past '\n'
    }

    false
}

// =============================================================================
// mkspell Argument Parsing (Phase 5)
// =============================================================================

/// Result of `rs_mkspell_output_fname`.
#[repr(C)]
pub struct MkspellFnameResult {
    /// NUL-terminated output filename (at most MAXPATHL bytes including NUL).
    pub fname: [u8; 4096],
    /// Length of the filename (excluding NUL).
    pub fname_len: u16,
    /// Whether the output is an .add.spl file (detected from filename).
    pub is_add: bool,
    /// Whether the output is an ascii .spl file (detected from filename).
    pub is_ascii: bool,
}

/// Compute the output `.spl` filename from `mkspell` arguments.
///
/// Logic mirrors the C code in `mkspell()`:
/// - `fcount == 1` and name ends in `.add`: output is `name.spl`
/// - `fcount == 1` (bare name): output is `name.<enc>.spl`
/// - name already ends in `.spl`: use as-is
/// - otherwise: output is `name.<enc>.spl`
///
/// `enc` is the encoding string (e.g. `"ascii"` or the result of `spell_enc()`),
/// passed from C as a NUL-terminated string.
///
/// Returns `0` on success, `-1` on error (NULL pointer or buffer overflow).
///
/// # Safety
/// All pointer parameters must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_mkspell_output_fname(
    fnames: *const *const u8,
    fcount: c_int,
    enc: *const u8,
    result_out: *mut MkspellFnameResult,
) -> c_int {
    if fnames.is_null() || enc.is_null() || result_out.is_null() || fcount < 1 {
        return -1;
    }

    let first = *fnames;
    if first.is_null() {
        return -1;
    }

    // Measure first filename length.
    let mut name_len = 0usize;
    while *first.add(name_len) != 0 {
        name_len += 1;
    }
    let name = std::slice::from_raw_parts(first, name_len);

    // Measure encoding length.
    let mut enc_len = 0usize;
    while *enc.add(enc_len) != 0 {
        enc_len += 1;
    }
    let enc_bytes = std::slice::from_raw_parts(enc, enc_len);

    let result = &mut *result_out;
    let buf = &mut result.fname;
    let mut out_len = 0usize;

    macro_rules! append {
        ($slice:expr) => {
            let s: &[u8] = $slice;
            if out_len + s.len() >= buf.len() {
                return -1;
            }
            buf[out_len..out_len + s.len()].copy_from_slice(s);
            out_len += s.len();
        };
    }

    if fcount == 1 && name_len > 4 && &name[name_len - 4..] == b".add" {
        // "path/en.latin1.add" -> "path/en.latin1.add.spl"
        append!(name);
        append!(b".spl");
    } else if fcount == 1 && name_len > 4 && &name[name_len - 4..] == b".spl" {
        // Already ends in ".spl" - use as-is
        append!(name);
    } else if name_len > 4 && &name[name_len - 4..] == b".spl" {
        // Multi-file case where first arg ends in ".spl"
        append!(name);
    } else {
        // Build "name.<enc>.spl"
        append!(name);
        append!(b".");
        append!(enc_bytes);
        append!(b".spl");
    }

    buf[out_len] = 0; // NUL-terminate

    // Detect .ascii. and .add. in the tail of the output filename.
    // Find path tail (last '/' or start).
    let fname_so_far = &buf[..out_len];
    let tail_start = fname_so_far
        .iter()
        .rposition(|&b| b == b'/')
        .map_or(0, |p| p + 1);
    let tail = &buf[tail_start..out_len];

    result.is_ascii = tail.windows(7).any(|w| w == b".ascii.");
    result.is_add = tail.windows(5).any(|w| w == b".add.");
    result.fname_len = out_len as u16;
    0
}

/// Validate `mkspell` input filenames and extract region names.
///
/// For `incount > 1`, each input filename must have a `_XX` suffix (two-char
/// region code). This function extracts those region chars into `region_name_out`
/// (a buffer of at least `incount * 2` bytes).
///
/// Returns:
/// - `0`: OK
/// - `1`: filename tail is too short to contain `_XX` suffix
/// - `2`: `incount > MAXREGIONS` (8)
/// - `-1`: NULL pointer or invalid arguments
///
/// When `incount == 1`, no region validation is needed and this function
/// returns `0` immediately.
///
/// # Safety
/// All pointer parameters must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_mkspell_validate_args(
    innames: *const *const u8,
    incount: c_int,
    region_name_out: *mut u8,
) -> c_int {
    const MAXREGIONS: i32 = 8;

    if innames.is_null() || region_name_out.is_null() || incount < 0 {
        return -1;
    }

    if incount > MAXREGIONS {
        return 2;
    }

    if incount <= 1 {
        return 0; // single input: no region validation needed
    }

    for i in 0..incount.unsigned_abs() as usize {
        let name_ptr = *innames.add(i);
        if name_ptr.is_null() {
            return -1;
        }

        // Measure length.
        let mut len = 0usize;
        while *name_ptr.add(len) != 0 {
            len += 1;
        }
        let name = std::slice::from_raw_parts(name_ptr, len);

        // Find the tail (after last '/').
        let tail_start = name.iter().rposition(|&b| b == b'/').map_or(0, |p| p + 1);
        let tail = &name[tail_start..];

        // Tail must be at least 5 chars: "ab_XX" (name + underscore + 2-char region)
        // and name[len-3] must be '_'.
        if tail.len() < 5 || name[len - 3] != b'_' {
            return 1;
        }

        // Extract region chars (lowercased).
        let r0 = name[len - 2].to_ascii_lowercase();
        let r1 = name[len - 1].to_ascii_lowercase();
        *region_name_out.add(i * 2) = r0;
        *region_name_out.add(i * 2 + 1) = r1;
    }

    0
}

// =============================================================================
// Phase 1 & 2: Functions migrated from C spellfile.c
// =============================================================================

extern "C" {
    // mb_ptr2char_adv: advance *pp by one multibyte char, return codepoint.
    // Matches C signature: int mb_cptr2char_adv(const char **pp)
    #[link_name = "mb_cptr2char_adv"]
    fn mb_ptr2char_adv_p(pp: *mut *const c_char) -> c_int;
    fn mb_charlen(p: *const c_char) -> c_int;
    fn emsg(s: *const c_char) -> bool;
    fn hash_init(ht: *mut crate::HashtabRaw);
    fn hash_hash(key: *const c_char) -> usize;
    fn hash_lookup(
        ht: *const crate::HashtabRaw,
        key: *const c_char,
        key_len: usize,
        hash: usize,
    ) -> *mut crate::HashitemRaw;
    fn hash_add_item(
        ht: *mut crate::HashtabRaw,
        hi: *mut crate::HashitemRaw,
        key: *mut c_char,
        hash: usize,
    );
    fn utf_char2len(c: c_int) -> c_int;
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn utf_ptr2len(p: *const c_char) -> c_int;
    #[link_name = "utfc_ptr2len"]
    fn utfc_ptr2len_spell(p: *const c_char) -> c_int;
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn ga_grow(gap: *mut crate::GArrayRaw, n: c_int);
    fn vim_regcomp(expr: *const c_char, re_flags: c_int) -> *mut std::ffi::c_void;
    #[link_name = "vim_regfree"]
    fn vim_regfree_spell(prog: *mut c_void);
    fn vim_regexec_prog(
        prog: *mut *mut c_void,
        ignore_case: bool,
        line: *const c_char,
        col: c_int,
    ) -> bool;
    fn hash_add(ht: *mut crate::HashtabRaw, key: *mut c_char);
    fn hash_clear(ht: *mut crate::HashtabRaw);
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize;
    #[link_name = "xmalloc"]
    fn xmalloc_spell(size: usize) -> *mut std::ffi::c_void;
    #[link_name = "xfree"]
    fn xfree_spell(ptr: *mut std::ffi::c_void);
}

extern "C" {
    // Global: bool did_set_spelltab (spell.c)
    #[link_name = "did_set_spelltab"]
    static did_set_spelltab_global: bool;

    // Global: spelltab_T spelltab (spell.c/spell.h)
    #[link_name = "spelltab"]
    static spelltab_global_sf: crate::SpelltabT;

    // Sentinel for removed hash items
    #[link_name = "hash_removed"]
    static hash_removed_sentinel: c_char;
}

/// Error message for word character conflict (E763).
static E763_MSG: &[u8] = b"E763: Word characters differ between spell files\0";
/// Error message for duplicate map entry (E783).
static E783_MSG: &[u8] = b"E783: Duplicate char in MAP entry\0";

/// Implements `set_spell_charflags` + `set_spell_finish` from spellfile.c.
///
/// Applies charflags and fold character mapping to the global spelltab.
/// Returns 0 (OK) on success, 1 (FAIL) if spelltab conflicts with a previously
/// loaded spell file.
///
/// # Safety
/// `flags_in` must be valid for `cnt` bytes. `fol` must be a valid NUL-terminated
/// multibyte string.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_set_spell_charflags(
    flags_in: *const u8,
    cnt: c_int,
    fol: *const c_char,
) -> c_int {
    // CF_WORD / CF_UPPER flags (from spellfile.c)
    const CF_WORD: u8 = 0x01;
    const CF_UPPER: u8 = 0x02;

    // Build a new spelltab_T on the stack (initialised to ASCII defaults via
    // the same logic as clear_spell_chartab).
    let mut new_st = crate::SpelltabT {
        st_isw: [false; 256],
        st_isu: [false; 256],
        st_fold: std::array::from_fn(|i| i as u8),
        st_upper: std::array::from_fn(|i| i as u8),
    };

    // Apply ASCII defaults.
    for i in b'0'..=b'9' {
        new_st.st_isw[i as usize] = true;
    }
    for i in b'A'..=b'Z' {
        new_st.st_isw[i as usize] = true;
        new_st.st_isu[i as usize] = true;
        new_st.st_fold[i as usize] = i + 0x20;
    }
    for i in b'a'..=b'z' {
        new_st.st_isw[i as usize] = true;
        new_st.st_upper[i as usize] = i - 0x20;
    }

    // Apply charflags for the 128..255 range (matching set_spell_charflags).
    let mut p = fol;
    for i in 0..128usize {
        if i < cnt as usize {
            let flag = *flags_in.add(i);
            new_st.st_isw[i + 128] = (flag & CF_WORD) != 0;
            new_st.st_isu[i + 128] = (flag & CF_UPPER) != 0;
        }

        if *p != 0 {
            let c = mb_ptr2char_adv_p(std::ptr::addr_of_mut!(p));
            new_st.st_fold[i + 128] = c as u8;
            if i + 128 != c as usize && new_st.st_isu[i + 128] && (c as usize) < 256 {
                new_st.st_upper[c as usize] = (i + 128) as u8;
            }
        }
    }

    // set_spell_finish: compare or install the new table.
    if did_set_spelltab_global {
        let st = std::ptr::addr_of!(spelltab_global_sf);
        for i in 0..256 {
            if (*st).st_isw[i] != new_st.st_isw[i]
                || (*st).st_isu[i] != new_st.st_isu[i]
                || (*st).st_fold[i] != new_st.st_fold[i]
                || (*st).st_upper[i] != new_st.st_upper[i]
            {
                let _ = emsg(E763_MSG.as_ptr().cast());
                return 1; // FAIL
            }
        }
    } else {
        // Copy new_st into the global spelltab.
        let st = std::ptr::addr_of!(spelltab_global_sf).cast_mut();
        *st = new_st;
        // Set did_set_spelltab = true.
        let flag = std::ptr::addr_of!(did_set_spelltab_global).cast_mut();
        *flag = true;
    }

    0 // OK
}

// =============================================================================
// Phase 1: rs_mb_str2wide -- replaces C mb_str2wide
// =============================================================================

/// Convert a multibyte string to a newly C-allocated wide-char (c_int) array.
///
/// Equivalent to the C `mb_str2wide` function.
/// The returned array is NUL-terminated and must be freed with `xfree`.
///
/// # Safety
/// `s` must be a valid NUL-terminated multibyte string.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_mb_str2wide(s: *const c_char) -> *mut c_int {
    let char_count = mb_charlen(s) as usize;
    let alloc_size = (char_count + 1) * std::mem::size_of::<c_int>();
    let res = xmalloc_spell(alloc_size).cast::<c_int>();

    let mut p = s;
    let mut i = 0usize;
    while *p != 0 {
        *res.add(i) = mb_ptr2char_adv_p(std::ptr::addr_of_mut!(p));
        i += 1;
    }
    *res.add(i) = 0; // NUL terminator

    res
}

// =============================================================================
// Phase 2: rs_set_sal_first -- replaces C set_sal_first
// =============================================================================

/// Fill the first-index table for soundalike (SAL) items.
///
/// Equivalent to the C `set_sal_first` function. Reorders `salitem_T` entries
/// in `sl_sal` so items with the same first wide-char low byte are contiguous,
/// then fills `sl_sal_first`.
///
/// # Safety
/// `slang` must be a valid pointer to a `SlangRaw` with `sl_sal` and
/// `sl_sal_first` properly initialised.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_set_sal_first(slang: *mut crate::SlangRaw) {
    let gap = std::ptr::addr_of_mut!((*slang).sl_sal);
    let sfirst = (*slang).sl_sal_first.as_mut_ptr();

    // Initialize all entries to -1.
    for i in 0..256usize {
        *sfirst.add(i) = -1;
    }

    let smp = (*gap).ga_data.cast::<crate::SalitemT>();
    let gap_len = (*gap).ga_len as usize;
    let mut i = 0usize;

    while i < gap_len {
        // Use the lowest byte of the first wide-char of sm_lead_w.
        let lead_w = (*smp.add(i)).sm_lead_w;
        let c = (*lead_w & 0xff) as usize;

        if *sfirst.add(c) == -1 {
            *sfirst.add(c) = i as c_int;

            // Skip over entries that already have the same index byte.
            while i + 1 < gap_len {
                let next_c = (*(*smp.add(i + 1)).sm_lead_w & 0xff) as usize;
                if next_c == c {
                    i += 1;
                } else {
                    break;
                }
            }

            // Move any further entries with the same byte to follow.
            let mut n = 1usize;
            while i + n < gap_len {
                let nc = (*(*smp.add(i + n)).sm_lead_w & 0xff) as usize;
                if nc == c {
                    // Move entry at i+n to position i+1, shifting others right.
                    i += 1;
                    // Save the entry we want to insert at position i.
                    let tsal = std::ptr::read(smp.add(i + n - 1));
                    // Shift smp[i .. i+n-1] right by one position.
                    std::ptr::copy(smp.add(i), smp.add(i + 1), n - 1);
                    std::ptr::write(smp.add(i), tsal);
                    n -= 1;
                } else {
                    n += 1;
                }
            }
        }
        i += 1;
    }
}

// =============================================================================
// Phase 2: rs_set_map_str -- replaces C set_map_str
// =============================================================================

/// Process MAP string, populating `sl_map_array` and `sl_map_hash`.
///
/// Equivalent to the C `set_map_str` function.
///
/// # Safety
/// `slang` must be a valid pointer to a `SlangRaw`. `map` must be a valid
/// NUL-terminated multibyte string.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_set_map_str(slang: *mut crate::SlangRaw, map: *const c_char) {
    if *map == 0 {
        (*slang).sl_has_map = false;
        return;
    }
    (*slang).sl_has_map = true;

    // Init the array empty.
    for i in 0..256usize {
        (*slang).sl_map_array[i] = 0;
    }
    hash_init(std::ptr::addr_of_mut!((*slang).sl_map_hash));

    let mut headc: c_int = 0;
    let mut p = map;

    while *p != 0 {
        let c = mb_ptr2char_adv_p(std::ptr::addr_of_mut!(p));

        if c == c_int::from(b'/') {
            headc = 0;
        } else {
            if headc == 0 {
                headc = c;
            }

            if c >= 256 {
                // Characters above 255: put in hash table.
                // Key layout: utf8(c) + NUL + utf8(headc) + NUL
                let cl = utf_char2len(c) as usize;
                let headcl = utf_char2len(headc) as usize;
                let b = xmalloc_spell(cl + headcl + 2).cast::<c_char>();
                utf_char2bytes(c, b);
                *b.add(cl) = 0;
                utf_char2bytes(headc, b.add(cl + 1));
                *b.add(cl + 1 + headcl) = 0;

                let hash = hash_hash(b);
                let key_len = libc::strlen(b);
                let hi = hash_lookup(std::ptr::addr_of!((*slang).sl_map_hash), b, key_len, hash);
                if hashitem_is_empty_sf(hi) {
                    hash_add_item(std::ptr::addr_of_mut!((*slang).sl_map_hash), hi, b, hash);
                } else {
                    let _ = emsg(E783_MSG.as_ptr().cast());
                    xfree_spell(b.cast());
                }
            } else {
                // ASCII/latin1: store in array.
                (*slang).sl_map_array[c as usize] = headc;
            }
        }
    }
}

/// Returns true if a hashitem is empty (null key or removed sentinel).
#[inline]
unsafe fn hashitem_is_empty_sf(hi: *const crate::HashitemRaw) -> bool {
    (*hi).hi_key.is_null() || std::ptr::eq((*hi).hi_key, std::ptr::addr_of!(hash_removed_sentinel))
}

/// Build SOFO character mapping tables on slang_T from from/to strings.
/// Replaces C set_sofo(). Uses sl_sal as a 256-entry int* array and sl_sal_first for latin1.
///
/// # Safety
/// - `slang` must be a valid non-null pointer to a SlangRaw.
/// - `from` and `to` must be valid NUL-terminated C strings.
#[no_mangle]
#[allow(clippy::cast_ptr_alignment)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_set_sofo(
    slang: *mut crate::SlangRaw,
    from: *const c_char,
    to: *const c_char,
) -> c_int {
    // Initialize sl_sal as a garray_T of int* with 256 slots.
    let gap = std::ptr::addr_of_mut!((*slang).sl_sal);
    (*gap).ga_len = 0;
    (*gap).ga_maxlen = 0;
    (*gap).ga_itemsize = std::mem::size_of::<*mut c_int>() as i32;
    (*gap).ga_growsize = 1;
    (*gap).ga_data = std::ptr::null_mut();
    ga_grow(gap, 256);
    std::ptr::write_bytes((*gap).ga_data, 0, std::mem::size_of::<*mut c_int>() * 256);
    (*gap).ga_len = 256;

    // First pass: count how many entries are needed per low-byte bucket (for chars >= 256).
    let mut p = from;
    let mut s = to;
    while *p != 0 && *s != 0 {
        let c = mb_ptr2char_adv_p(std::ptr::addr_of_mut!(p));
        s = s.add(utf_ptr2len(s) as usize);
        if c >= 256 {
            let bucket = (c & 0xff) as usize;
            let slot = ((*gap).ga_data as *mut *mut c_int).add(bucket);
            let cur = *slot as usize;
            *slot = (cur + 1) as *mut c_int; // temp: count
        }
    }
    if *p != 0 || *s != 0 {
        // lengths differ
        return -1; // SP_FORMERROR
    }

    // Allocate int arrays for each bucket that has entries.
    for i in 0..256usize {
        let slot = ((*gap).ga_data as *mut *mut c_int).add(i);
        let count = *slot as usize;
        if count > 0 {
            let arr = xmalloc_spell(std::mem::size_of::<c_int>() * (count * 2 + 1)).cast::<c_int>();
            *arr = 0; // NUL terminator
            *slot = arr;
        }
    }

    // Clear sl_sal_first and do second pass to fill mappings.
    (*slang).sl_sal_first = [0i32; 256];
    let mut p = from;
    let mut s = to;
    while *p != 0 && *s != 0 {
        let c = mb_ptr2char_adv_p(std::ptr::addr_of_mut!(p));
        let target = mb_ptr2char_adv_p(std::ptr::addr_of_mut!(s));
        if c >= 256 {
            let bucket = (c & 0xff) as usize;
            let slot = ((*gap).ga_data as *mut *mut c_int).add(bucket);
            let mut inp = *slot;
            while *inp != 0 {
                inp = inp.add(1);
            }
            *inp = c;
            inp = inp.add(1);
            *inp = target;
            inp = inp.add(1);
            *inp = 0; // NUL terminator
        } else {
            (*slang).sl_sal_first[c as usize] = target;
        }
    }
    0 // OK
}

// Size of spell block in bytes (matches SBLOCKSIZE in spellfile.c).
// Used as i32 for msm calculation arithmetic.
const SBLOCKSIZE_I32: i32 = 16000;

/// Parse integer digits from a byte slice, advancing the slice past the digits.
/// Returns None if no digits are present.
fn parse_digits(s: &[u8]) -> Option<(i32, &[u8])> {
    if s.is_empty() || !s[0].is_ascii_digit() {
        return None;
    }
    let mut val: i32 = 0;
    let mut i = 0;
    while i < s.len() && s[i].is_ascii_digit() {
        val = val
            .saturating_mul(10)
            .saturating_add(i32::from(s[i] - b'0'));
        i += 1;
    }
    Some((val, &s[i..]))
}

/// Validate and parse the 'mkspellmem' option string, returning (start, incr, added).
/// Returns None if the string is invalid.
fn parse_msm(msm: &[u8]) -> Option<(i32, i32, i32)> {
    let (raw_start, rest) = parse_digits(msm)?;
    let rest = rest.strip_prefix(b",")?;
    let (raw_incr, rest) = parse_digits(rest)?;
    let rest = rest.strip_prefix(b",")?;
    let (raw_added, rest) = parse_digits(rest)?;
    if !rest.is_empty() {
        return None;
    }
    let start = (raw_start * 10) / (SBLOCKSIZE_I32 / 102);
    let incr = (raw_incr * 102) / (SBLOCKSIZE_I32 / 10);
    let added = raw_added * 1024;
    if start == 0 || incr == 0 || added == 0 || incr > start {
        return None;
    }
    Some((start, incr, added))
}

/// Check the 'mkspellmem' option and return parsed (start, incr, added) values via out-params.
/// Returns 0 (OK) on success, 1 (FAIL) on error.
///
/// # Safety
/// - `msm` must be a valid NUL-terminated C string.
/// - `start_out`, `incr_out`, `added_out` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_check_msm(
    msm: *const c_char,
    start_out: *mut c_int,
    incr_out: *mut c_int,
    added_out: *mut c_int,
) -> c_int {
    let len = libc::strlen(msm);
    let bytes = std::slice::from_raw_parts(msm.cast::<u8>(), len);
    match parse_msm(bytes) {
        Some((start, incr, added)) => {
            *start_out = start;
            *incr_out = incr;
            *added_out = added;
            0 // OK
        }
        None => 1, // FAIL
    }
}

extern "C" {
    #[link_name = "p_msm"]
    static p_msm_global: *const c_char;
}

// Compression tuning globals (moved from C spellfile.c).
// Defaults match the C definitions: compress_start=30000, compress_inc=100, compress_added=500000.
static mut COMPRESS_START: c_int = 30_000;
static mut COMPRESS_INC: c_int = 100;
static mut COMPRESS_ADDED: c_int = 500_000;

/// Rust replacement for the C `spell_check_msm()` function.
/// Parses the 'mkspellmem' option and sets the compression globals.
/// Returns 0 (OK) on success, 1 (FAIL) on error.
///
/// This replaces the C thin-wrapper that called `rs_spell_check_msm` then set globals.
///
/// # Safety
/// Called from C; no raw pointer arguments.
#[export_name = "spell_check_msm"]
pub unsafe extern "C" fn rs_do_spell_check_msm() -> c_int {
    let msm = p_msm_global;
    if msm.is_null() {
        return 1; // FAIL
    }
    let len = libc::strlen(msm);
    let bytes = std::slice::from_raw_parts(msm.cast::<u8>(), len);
    match parse_msm(bytes) {
        Some((start, incr, added)) => {
            COMPRESS_START = start;
            COMPRESS_INC = incr;
            COMPRESS_ADDED = added;
            0 // OK
        }
        None => 1, // FAIL
    }
}

// =============================================================================
// Phase 2: Spell Section Writing (write_vim_spell helper)
// =============================================================================

/// Section flags: section is required for correct spell checking.
const SNF_REQUIRED_U8: u8 = 1;

/// Write a simple string section (SN_INFO, SN_MIDWORD, SN_SYLLABLE).
/// Returns bytes written (0 if s is null).
unsafe fn write_cstr_section(buf: &mut Vec<u8>, section_id: u8, flags: u8, s: *const c_char) {
    if s.is_null() {
        return;
    }
    let len = libc::strlen(s);
    if len == 0 {
        return;
    }
    // section header: id + flags + len(4 bytes BE)
    buf.push(section_id);
    buf.push(flags);
    buf.extend_from_slice(&(len as u32).to_be_bytes());
    // section data
    let data = std::slice::from_raw_parts(s.cast::<u8>(), len);
    buf.extend_from_slice(data);
}

/// Write a fromto (REP/SAL/REPSAL) section to buf.
/// For SAL sections, sal_flags is non-zero.
unsafe fn write_fromto_section(
    buf: &mut Vec<u8>,
    section_id: u8,
    sal_flags: u8,
    from_ptrs: *const *const u8,
    to_ptrs: *const *const u8,
    count: usize,
) {
    if count == 0 {
        return;
    }
    let has_sal_header = sal_flags != 0;

    // Compute section length
    let mut data_len: usize = 2; // count (2 bytes)
    if has_sal_header {
        data_len += 1; // salflags (1 byte)
    }
    for i in 0..count {
        let from = *from_ptrs.add(i);
        let to = *to_ptrs.add(i);
        let flen = libc::strlen(from.cast::<c_char>());
        let tlen = libc::strlen(to.cast::<c_char>());
        data_len += 1 + flen + 1 + tlen;
    }

    // Write section header
    buf.push(section_id);
    buf.push(0); // sectionflags: not required
    buf.extend_from_slice(&(data_len as u32).to_be_bytes());

    // Write SAL flags if applicable
    if has_sal_header {
        buf.push(sal_flags);
    }

    // Write count (2 bytes BE)
    buf.extend_from_slice(&(count as u16).to_be_bytes());

    // Write each from-to pair
    for i in 0..count {
        let from = *from_ptrs.add(i);
        let to = *to_ptrs.add(i);
        let flen = libc::strlen(from.cast::<c_char>());
        let tlen = libc::strlen(to.cast::<c_char>());
        buf.push(flen as u8);
        if flen > 0 {
            buf.extend_from_slice(std::slice::from_raw_parts(from, flen));
        }
        buf.push(tlen as u8);
        if tlen > 0 {
            buf.extend_from_slice(std::slice::from_raw_parts(to, tlen));
        }
    }
}

/// Parameters for writing spell sections.
/// All fields correspond to spellinfo_T fields, extracted by the C caller.
#[repr(C)]
pub struct SpellSectionParams {
    /// SN_INFO: info text (nullable)
    pub si_info: *const c_char,

    /// SN_REGION: region count (0 or 1 = no region section)
    pub si_region_count: c_int,
    /// SN_REGION: region name bytes (si_region_count * 2 bytes)
    pub si_region_name: *const u8,

    /// SN_CHARFLAGS: skip if true (ascii mode or add file)
    pub si_skip_charflags: bool,

    /// SN_MIDWORD: midword chars (nullable)
    pub si_midword: *const c_char,

    /// SN_PREFCOND: condition strings
    pub si_prefcond_strs: *const *const u8,
    pub si_prefcond_count: c_int,

    /// SN_REP (pre-sorted by caller)
    pub si_rep_from: *const *const u8,
    pub si_rep_to: *const *const u8,
    pub si_rep_count: c_int,

    /// SN_SAL: use SAL section (vs SOFO)
    pub si_use_sal: bool,
    pub si_sal_from: *const *const u8,
    pub si_sal_to: *const *const u8,
    pub si_sal_count: c_int,
    /// SAL flags: combination of SAL_F0LLOWUP, SAL_COLLAPSE, SAL_REM_ACCENTS
    pub si_sal_flags: u8,

    /// SN_REPSAL (pre-sorted by caller)
    pub si_repsal_from: *const *const u8,
    pub si_repsal_to: *const *const u8,
    pub si_repsal_count: c_int,

    /// SN_SOFO: soundfold from/to (nullable; only written if not using SAL)
    pub si_sofofr: *const c_char,
    pub si_sofoto: *const c_char,

    /// SN_MAP: map string data
    pub si_map_data: *const u8,
    pub si_map_len: c_int,

    /// SN_SUGFILE: timestamp (0 = don't write)
    pub si_sugtime: i64,

    /// SN_NOSPLITSUGS
    pub si_nosplitsugs: bool,

    /// SN_NOCOMPOUNDSUGS
    pub si_nocompoundsugs: bool,

    /// SN_COMPOUND: compound flags (nullable = no compound section)
    pub si_compflags: *mut c_char,
    pub si_compmax: u8,
    pub si_compminlen: u8,
    pub si_compsylmax: u8,
    pub si_compoptions: u16,
    pub si_comppat_strs: *const *const u8,
    pub si_comppat_count: c_int,

    /// SN_NOBREAK
    pub si_nobreak: bool,

    /// SN_SYLLABLE: syllable string (nullable)
    pub si_syllable: *const c_char,
}

/// Write all spell file sections (except SN_CHARFLAGS, SN_WORDS, and tree data)
/// to the output buffer.
///
/// The caller is responsible for:
/// - Writing the file header (VIMSPELLMAGIC + version) before calling this
/// - Writing SN_CHARFLAGS section (needs spelltab)
/// - Writing SN_WORDS section (needs hashtable iteration)
/// - Writing tree data (LWORDTREE, KWORDTREE, PREFIXTREE) after sections
/// - Writing the final 0 byte error check
///
/// Returns the number of bytes written to buf, or -1 on error.
///
/// # Safety
/// All pointers in params must be valid for the duration of the call.
/// buf must be a valid writable buffer of buf_len bytes.
#[no_mangle]
#[allow(clippy::too_many_lines, clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_write_spell_sections(
    params: *const SpellSectionParams,
    buf: *mut u8,
    buf_len: usize,
    written_out: *mut usize,
) -> c_int {
    if params.is_null() || buf.is_null() || written_out.is_null() {
        return SP_OTHERERROR;
    }
    let p = &*params;
    let mut out: Vec<u8> = Vec::with_capacity(4096);

    // SN_INFO: <infotext>
    if !p.si_info.is_null() {
        let len = libc::strlen(p.si_info);
        if len > 0 {
            out.push(SN_INFO);
            out.push(0); // sectionflags
            out.extend_from_slice(&(len as u32).to_be_bytes());
            out.extend_from_slice(std::slice::from_raw_parts(p.si_info.cast::<u8>(), len));
        }
    }

    // SN_REGION: <regionname> ...
    if p.si_region_count > 1 {
        let region_len = (p.si_region_count as usize) * 2;
        out.push(SN_REGION);
        out.push(SNF_REQUIRED_U8);
        out.extend_from_slice(&(region_len as u32).to_be_bytes());
        out.extend_from_slice(std::slice::from_raw_parts(p.si_region_name, region_len));
    }

    // SN_CHARFLAGS is written by C (needs spelltab + utf_char2bytes)
    // Placeholder: caller inserts it between SN_REGION and SN_MIDWORD.

    // SN_MIDWORD: <midword>
    write_cstr_section(&mut out, SN_MIDWORD, SNF_REQUIRED_U8, p.si_midword);

    // SN_PREFCOND: <prefcondcnt> <prefcond> ...
    if p.si_prefcond_count > 0 && !p.si_prefcond_strs.is_null() {
        let count = p.si_prefcond_count as usize;
        // Compute data length: 2 bytes for count + sum of (1 + len) for each
        let mut data_len: usize = 2;
        for i in 0..count {
            let s = *p.si_prefcond_strs.add(i);
            let slen = if s.is_null() {
                0
            } else {
                libc::strlen(s.cast::<c_char>())
            };
            data_len += 1 + slen;
        }
        out.push(SN_PREFCOND);
        out.push(SNF_REQUIRED_U8);
        out.extend_from_slice(&(data_len as u32).to_be_bytes());
        // Write count (2 bytes BE)
        out.extend_from_slice(&(count as u16).to_be_bytes());
        for i in 0..count {
            let s = *p.si_prefcond_strs.add(i);
            let slen = if s.is_null() {
                0
            } else {
                libc::strlen(s.cast::<c_char>())
            };
            out.push(slen as u8);
            if slen > 0 {
                out.extend_from_slice(std::slice::from_raw_parts(s, slen));
            }
        }
    }

    // SN_REP: <repcount> <rep> ...
    if p.si_rep_count > 0 && !p.si_rep_from.is_null() {
        write_fromto_section(
            &mut out,
            SN_REP,
            0,
            p.si_rep_from,
            p.si_rep_to,
            p.si_rep_count as usize,
        );
    }

    // SN_SAL or SN_SOFO
    if p.si_use_sal {
        // SN_SAL: <salflags> <salcount> <sal> ...
        if p.si_sal_count > 0 && !p.si_sal_from.is_null() {
            write_fromto_section(
                &mut out,
                SN_SAL,
                p.si_sal_flags,
                p.si_sal_from,
                p.si_sal_to,
                p.si_sal_count as usize,
            );
        }
    } else if !p.si_sofofr.is_null() && !p.si_sofoto.is_null() {
        // SN_SOFO: <sofofromlen> <sofofrom> <sofotolen> <sofoto>
        let flen = libc::strlen(p.si_sofofr);
        let tlen = libc::strlen(p.si_sofoto);
        let data_len = 2 + flen + 2 + tlen;
        out.push(SN_SOFO);
        out.push(0);
        out.extend_from_slice(&(data_len as u32).to_be_bytes());
        out.extend_from_slice(&(flen as u16).to_be_bytes());
        out.extend_from_slice(std::slice::from_raw_parts(p.si_sofofr.cast::<u8>(), flen));
        out.extend_from_slice(&(tlen as u16).to_be_bytes());
        out.extend_from_slice(std::slice::from_raw_parts(p.si_sofoto.cast::<u8>(), tlen));
    }

    // SN_REPSAL: <repcount> <rep> ...
    if p.si_repsal_count > 0 && !p.si_repsal_from.is_null() {
        write_fromto_section(
            &mut out,
            SN_REPSAL,
            0,
            p.si_repsal_from,
            p.si_repsal_to,
            p.si_repsal_count as usize,
        );
    }

    // SN_WORDS is written by C (needs hashtable iteration)

    // SN_MAP: <mapstr>
    if p.si_map_len > 0 && !p.si_map_data.is_null() {
        let mlen = p.si_map_len as usize;
        out.push(SN_MAP);
        out.push(0);
        out.extend_from_slice(&(mlen as u32).to_be_bytes());
        out.extend_from_slice(std::slice::from_raw_parts(p.si_map_data, mlen));
    }

    // SN_SUGFILE: <timestamp>
    if p.si_sugtime != 0 {
        out.push(SN_SUGFILE);
        out.push(0);
        out.extend_from_slice(&8u32.to_be_bytes()); // section len = 8
        out.extend_from_slice(&p.si_sugtime.to_be_bytes());
    }

    // SN_NOSPLITSUGS: nothing (presence is what matters)
    if p.si_nosplitsugs {
        out.push(SN_NOSPLITSUGS);
        out.push(0);
        out.extend_from_slice(&0u32.to_be_bytes());
    }

    // SN_NOCOMPOUNDSUGS: nothing
    if p.si_nocompoundsugs {
        out.push(SN_NOCOMPOUNDSUGS);
        out.push(0);
        out.extend_from_slice(&0u32.to_be_bytes());
    }

    // SN_COMPOUND
    if !p.si_compflags.is_null() {
        let cflen = libc::strlen(p.si_compflags);
        let npat = p.si_comppat_count as usize;
        // Fixed overhead: max(1)+minlen(1)+sylmax(1)+compat0(1)+compoptions(1)+patcount(2) = 7
        // Note: compoptions is written as compat0 byte then low byte (2 bytes total),
        // matching C's: putc(0, fd); putc(compoptions, fd);
        let mut data_len: usize = 1 + 1 + 1 + 1 + 1 + 2; // max,minlen,sylmax,compat0,compoptions,patcount
        data_len += cflen; // compflags
        for i in 0..npat {
            let s = *p.si_comppat_strs.add(i);
            let slen = if s.is_null() {
                0
            } else {
                libc::strlen(s.cast::<c_char>())
            };
            data_len += 1 + slen;
        }
        out.push(SN_COMPOUND);
        out.push(0);
        out.extend_from_slice(&(data_len as u32).to_be_bytes());
        out.push(p.si_compmax);
        out.push(p.si_compminlen);
        out.push(p.si_compsylmax);
        out.push(0); // Vim 7.0b compatibility
                     // compoptions: 1 byte (low byte only, matching C's putc(compoptions, fd))
        out.push((p.si_compoptions & 0xFF) as u8);
        // comppatcount: 2 bytes BE
        out.extend_from_slice(&(npat as u16).to_be_bytes());
        for i in 0..npat {
            let s = *p.si_comppat_strs.add(i);
            let slen = if s.is_null() {
                0
            } else {
                libc::strlen(s.cast::<c_char>())
            };
            out.push(slen as u8);
            if slen > 0 {
                out.extend_from_slice(std::slice::from_raw_parts(s, slen));
            }
        }
        // compflags
        out.extend_from_slice(std::slice::from_raw_parts(
            p.si_compflags.cast::<u8>(),
            cflen,
        ));
    }

    // SN_NOBREAK: nothing (presence is what matters)
    if p.si_nobreak {
        out.push(SN_NOBREAK);
        out.push(0);
        out.extend_from_slice(&0u32.to_be_bytes());
    }

    // SN_SYLLABLE: <syllable>
    write_cstr_section(&mut out, SN_SYLLABLE, 0, p.si_syllable);

    // Note: SN_CHARFLAGS, SN_WORDS, and SN_END are written by the C caller.

    // Copy to output buffer
    if out.len() > buf_len {
        return SP_TRUNCERROR;
    }
    let out_slice = std::slice::from_raw_parts_mut(buf, out.len());
    out_slice.copy_from_slice(&out);
    *written_out = out.len();
    0
}

// =============================================================================
// Phase 1: repr(C) struct for wordnode_T -- direct field access, no accessors
// =============================================================================

/// Union for wn_u1 field: hashkey (during compression) or index (after first pass).
/// C layout: union { uint8_t hashkey[6]; int index; }
/// sizeof = 8 (6 rounded up to 4-alignment), alignof = 4.
#[repr(C)]
pub union WordnodeU1 {
    pub hashkey: [u8; 6],
    pub index: c_int,
    _pad: [u8; 8],
}

/// Union for wn_u2 field: next node (compression) or wnode parent (serialization).
/// C layout: union { wordnode_T *next; wordnode_T *wnode; }
/// sizeof = 8, alignof = 8.
#[repr(C)]
pub union WordnodeU2 {
    pub next: *mut WordnodeT,
    pub wnode: *mut WordnodeT,
}

/// Full repr(C) mirror of C `wordnode_T` (spellfile.c lines 596-623).
#[allow(non_snake_case)]
///
/// Layout (64-bit):
///   offset  0: wn_u1   (8 bytes, union of hashkey[6]/index)
///   offset  8: wn_u2   (8 bytes, union of next*/wnode*)
///   offset 16: wn_child   (8 bytes)
///   offset 24: wn_sibling (8 bytes)
///   offset 32: wn_refs    (4 bytes, c_int)
///   offset 36: wn_byte    (1 byte, u8)
///   offset 37: wn_affixID (1 byte, u8)
///   offset 38: wn_flags   (2 bytes, u16)
///   offset 40: wn_region  (2 bytes, i16)
///   offset 42: _pad       (6 bytes)
///   sizeof = 48, alignof = 8
#[repr(C)]
pub struct WordnodeT {
    pub wn_u1: WordnodeU1,
    pub wn_u2: WordnodeU2,
    pub wn_child: *mut WordnodeT,
    pub wn_sibling: *mut WordnodeT,
    pub wn_refs: c_int,
    pub wn_byte: u8,
    pub wn_affixID: u8,
    pub wn_flags: u16,
    pub wn_region: i16,
    _pad: [u8; 6],
}

// Compile-time layout assertions for WordnodeT.
const _: () = {
    assert!(std::mem::size_of::<WordnodeT>() == 48);
    assert!(std::mem::align_of::<WordnodeT>() == 8);
    assert!(std::mem::offset_of!(WordnodeT, wn_u1) == 0);
    assert!(std::mem::offset_of!(WordnodeT, wn_u2) == 8);
    assert!(std::mem::offset_of!(WordnodeT, wn_child) == 16);
    assert!(std::mem::offset_of!(WordnodeT, wn_sibling) == 24);
    assert!(std::mem::offset_of!(WordnodeT, wn_refs) == 32);
    assert!(std::mem::offset_of!(WordnodeT, wn_byte) == 36);
    assert!(std::mem::offset_of!(WordnodeT, wn_affixID) == 37);
    assert!(std::mem::offset_of!(WordnodeT, wn_flags) == 38);
    assert!(std::mem::offset_of!(WordnodeT, wn_region) == 40);
};

// =============================================================================
// Phase 2: repr(C) structs for sblock_T and spellinfo_T -- direct field access
// =============================================================================

/// sblock_T header: the flexible array `sb_data[]` follows immediately after.
///
/// C layout (64-bit):
///   offset  0: sb_used  (4 bytes, c_int)
///   offset  4: _pad     (4 bytes)
///   offset  8: sb_next  (8 bytes, *mut SblockT)
///   sizeof header = 16, flexible sb_data[] starts at offset 16.
/// SBLOCKSIZE = 16000 bytes for sb_data
#[repr(C)]
pub struct SblockT {
    pub sb_used: c_int,
    _pad: [u8; 4],
    pub sb_next: *mut SblockT,
    // sb_data[] flexible array follows; accessed via pointer arithmetic
}

/// Size of the sb_data[] region in each sblock.
pub const SBLOCKSIZE: usize = 16000;

// Compile-time layout assertions for SblockT.
const _: () = {
    assert!(std::mem::size_of::<SblockT>() == 16);
    assert!(std::mem::align_of::<SblockT>() == 8);
    assert!(std::mem::offset_of!(SblockT, sb_used) == 0);
    assert!(std::mem::offset_of!(SblockT, sb_next) == 8);
};

/// Opaque vimconv_T: 24 bytes, not accessed from Rust.
/// Layout: int vc_type, int vc_factor, iconv_t vc_fd (8 bytes), bool vc_fail + 7 pad.
type VimconvOpaque = [u8; 24];

/// Full repr(C) mirror of C `spellinfo_T` (spellfile.c lines 655-720).
///
/// sizeof = 736, alignof = 8 (on 64-bit Linux).
///
/// Key offsets (verified by /tmp/check_offsets):
///   0: si_foldroot, 8: si_foldwcount, 16: si_keeproot, 24: si_keepwcount
///   32: si_prefroot, 40: si_sugtree, 48: si_blocks, 56: si_blocks_cnt
///   60: si_did_emsg, 64: si_compress_cnt, 72: si_first_free, 80: si_free_count
///   88: si_spellbuf, 96: si_ascii, 112: si_conv (24 bytes), 140: si_verbose
///   144: si_msg_count, 152: si_info, 160: si_region_count, 164: si_region_name
///   184: si_rep, 208: si_repsal, 232: si_sal, 256: si_sofofr, 296: si_commonwords
///   592: si_sugtime, 600: si_rem_accents, 608: si_map, 632: si_midword
///   680: si_compflags, 696: si_syllable, 704: si_prefcond, 728: si_newprefID, 732: si_newcompID
#[repr(C)]
#[allow(non_snake_case)]
pub struct SpellinfoT {
    pub si_foldroot: *mut WordnodeT,       // offset 0
    pub si_foldwcount: c_int,              // offset 8
    _pad0: [u8; 4],                        // offset 12 (pad to align si_keeproot)
    pub si_keeproot: *mut WordnodeT,       // offset 16
    pub si_keepwcount: c_int,              // offset 24
    _pad1: [u8; 4],                        // offset 28
    pub si_prefroot: *mut WordnodeT,       // offset 32
    pub si_sugtree: c_int,                 // offset 40
    _pad2: [u8; 4],                        // offset 44
    pub si_blocks: *mut SblockT,           // offset 48
    pub si_blocks_cnt: c_int,              // offset 56
    pub si_did_emsg: c_int,                // offset 60
    pub si_compress_cnt: c_int,            // offset 64
    _pad3: [u8; 4],                        // offset 68
    pub si_first_free: *mut WordnodeT,     // offset 72
    pub si_free_count: c_int,              // offset 80
    _pad4: [u8; 4],                        // offset 84
    pub si_spellbuf: *mut c_void,          // offset 88
    pub si_ascii: c_int,                   // offset 96
    pub si_add: c_int,                     // offset 100
    pub si_clear_chartab: c_int,           // offset 104
    pub si_region: c_int,                  // offset 108
    si_conv: VimconvOpaque,                // offset 112 (24 bytes, opaque)
    pub si_memtot: c_int,                  // offset 136
    pub si_verbose: c_int,                 // offset 140
    pub si_msg_count: c_int,               // offset 144
    _pad5: [u8; 4],                        // offset 148
    pub si_info: *mut c_char,              // offset 152
    pub si_region_count: c_int,            // offset 160
    pub si_region_name: [c_char; 17],      // offset 164 (MAXREGIONS*2+1 = 17)
    _pad6: [u8; 3],                        // offset 181 (pad to 8-align si_rep)
    pub si_rep: crate::GArrayRaw,          // offset 184 (24 bytes)
    pub si_repsal: crate::GArrayRaw,       // offset 208 (24 bytes)
    pub si_sal: crate::GArrayRaw,          // offset 232 (24 bytes)
    pub si_sofofr: *mut c_char,            // offset 256
    pub si_sofoto: *mut c_char,            // offset 264
    pub si_nosugfile: c_int,               // offset 272
    pub si_nosplitsugs: c_int,             // offset 276
    pub si_nocompoundsugs: c_int,          // offset 280
    pub si_followup: c_int,                // offset 284
    pub si_collapse: c_int,                // offset 288
    _pad7: [u8; 4],                        // offset 292
    pub si_commonwords: crate::HashtabRaw, // offset 296 (296 bytes)
    pub si_sugtime: i64,                   // offset 592 (time_t = 8 bytes)
    pub si_rem_accents: c_int,             // offset 600
    _pad8: [u8; 4],                        // offset 604
    pub si_map: crate::GArrayRaw,          // offset 608 (24 bytes)
    pub si_midword: *mut c_char,           // offset 632
    pub si_compmax: c_int,                 // offset 640
    pub si_compminlen: c_int,              // offset 644
    pub si_compsylmax: c_int,              // offset 648
    pub si_compoptions: c_int,             // offset 652
    pub si_comppat: crate::GArrayRaw,      // offset 656 (24 bytes)
    pub si_compflags: *mut c_char,         // offset 680
    pub si_nobreak: c_char,                // offset 688
    _pad9: [u8; 7],                        // offset 689
    pub si_syllable: *mut c_char,          // offset 696
    pub si_prefcond: crate::GArrayRaw,     // offset 704 (24 bytes)
    pub si_newprefID: c_int,               // offset 728
    pub si_newcompID: c_int,               // offset 732
}

// Compile-time layout assertions for SpellinfoT.
const _: () = {
    assert!(std::mem::size_of::<SpellinfoT>() == 736);
    assert!(std::mem::align_of::<SpellinfoT>() == 8);
    assert!(std::mem::offset_of!(SpellinfoT, si_foldroot) == 0);
    assert!(std::mem::offset_of!(SpellinfoT, si_foldwcount) == 8);
    assert!(std::mem::offset_of!(SpellinfoT, si_keeproot) == 16);
    assert!(std::mem::offset_of!(SpellinfoT, si_sugtree) == 40);
    assert!(std::mem::offset_of!(SpellinfoT, si_blocks) == 48);
    assert!(std::mem::offset_of!(SpellinfoT, si_blocks_cnt) == 56);
    assert!(std::mem::offset_of!(SpellinfoT, si_did_emsg) == 60);
    assert!(std::mem::offset_of!(SpellinfoT, si_compress_cnt) == 64);
    assert!(std::mem::offset_of!(SpellinfoT, si_first_free) == 72);
    assert!(std::mem::offset_of!(SpellinfoT, si_free_count) == 80);
    assert!(std::mem::offset_of!(SpellinfoT, si_verbose) == 140);
    assert!(std::mem::offset_of!(SpellinfoT, si_msg_count) == 144);
    assert!(std::mem::offset_of!(SpellinfoT, si_region_name) == 164);
    assert!(std::mem::offset_of!(SpellinfoT, si_rep) == 184);
    assert!(std::mem::offset_of!(SpellinfoT, si_commonwords) == 296);
    assert!(std::mem::offset_of!(SpellinfoT, si_sugtime) == 592);
    assert!(std::mem::offset_of!(SpellinfoT, si_map) == 608);
    assert!(std::mem::offset_of!(SpellinfoT, si_compflags) == 680);
    assert!(std::mem::offset_of!(SpellinfoT, si_syllable) == 696);
    assert!(std::mem::offset_of!(SpellinfoT, si_prefcond) == 704);
    assert!(std::mem::offset_of!(SpellinfoT, si_newprefID) == 728);
    assert!(std::mem::offset_of!(SpellinfoT, si_newcompID) == 732);
};

/// Flag indicating a prefix tree NUL node with no flags.
const PFX_FLAGS: i32 = -256;

/// BY_NOFLAGS, BY_INDEX, BY_FLAGS, BY_FLAGS2 values (match C enum).
const BY_NOFLAGS: u8 = 0;
const BY_INDEX: u8 = 1;
const BY_FLAGS_VAL: u8 = 2;
const BY_FLAGS2: u8 = 3;

/// Spell word flags used in .spl tree nodes (from spell_defs.h).
const WF_REGION: c_int = 0x01; // region byte follows
const WF_AFX: c_int = 0x20; // affix ID follows

/// Recursively serialize a word tree node into `out`.
///
/// Mirrors the C `put_node` function exactly.
///
/// # Safety
/// `node` must be a valid (or null) wordnode_T pointer from C.
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
unsafe fn put_node_inner(
    node: *mut WordnodeT,
    out: &mut Vec<u8>,
    idx: c_int,
    regionmask: c_int,
    prefixtree: bool,
) -> c_int {
    if node.is_null() {
        return idx;
    }

    // Store the index where this node starts.
    (*node).wn_u1.index = idx;

    // Count siblings.
    let mut siblingcount: c_int = 0;
    let mut np = node;
    while !np.is_null() {
        siblingcount += 1;
        np = (*np).wn_sibling;
    }

    // Write <siblingcount>
    out.push(siblingcount as u8);

    // Write each sibling byte and optional extra info.
    np = node;
    while !np.is_null() {
        let byte = (*np).wn_byte;
        if byte == 0 {
            // NUL byte: end of word. Write flags.
            if prefixtree {
                let flags = (*np).wn_flags;
                if flags == PFX_FLAGS as u16 {
                    out.push(BY_NOFLAGS); // <byte>
                } else {
                    out.push(BY_FLAGS_VAL); // <byte>
                    out.push(flags as u8); // <pflags>
                }
                out.push((*np).wn_affixID); // <affixID>
                                            // <prefcondnr> in 2 bytes BE
                let region = (*np).wn_region as u16;
                out.extend_from_slice(&region.to_be_bytes());
            } else {
                let wn_flags = (*np).wn_flags as c_int;
                let wn_region = (*np).wn_region as c_int;
                let wn_affixid = (*np).wn_affixID as c_int;
                let mut flags = wn_flags;
                if regionmask != 0 && wn_region != regionmask {
                    flags |= WF_REGION;
                }
                if wn_affixid != 0 {
                    flags |= WF_AFX;
                }
                if flags == 0 {
                    out.push(BY_NOFLAGS); // <byte>
                } else if wn_flags >= 0x100 {
                    out.push(BY_FLAGS2); // <byte>
                    out.push(flags as u8); // <flags>
                    out.push(((flags as u32) >> 8) as u8); // <flags2>
                } else {
                    out.push(BY_FLAGS_VAL); // <byte>
                    out.push(flags as u8); // <flags>
                }
                if flags & WF_REGION != 0 {
                    out.push(wn_region as u8); // <region>
                }
                if flags & WF_AFX != 0 {
                    out.push(wn_affixid as u8); // <affixID>
                }
            }
        } else {
            let child = (*np).wn_child;
            let child_index = (*child).wn_u1.index;
            let child_wnode = (*child).wn_u2.wnode;

            if child_index != 0 && child_wnode != node {
                // Child was written elsewhere; write a reference.
                out.push(BY_INDEX); // <byte>
                                    // <nodeidx> in 3 bytes BE
                out.push(((child_index as u32) >> 16) as u8);
                out.push(((child_index as u32) >> 8) as u8);
                out.push((child_index as u32) as u8);
            } else if child_wnode.is_null() {
                // We will write the child below; claim it.
                (*child).wn_u2.wnode = node;
            }

            out.push(byte); // <byte> or <xbyte>
        }
        np = (*np).wn_sibling;
    }

    // Space used when reading: one per sibling plus one for the count.
    let mut newindex = idx + siblingcount + 1;

    // Recursively write children.
    np = node;
    while !np.is_null() {
        let byte = (*np).wn_byte;
        if byte != 0 {
            let child = (*np).wn_child;
            if (*child).wn_u2.wnode == node {
                newindex = put_node_inner(child, out, newindex, regionmask, prefixtree);
            }
        }
        np = (*np).wn_sibling;
    }

    newindex
}

/// Recursively clear the index and wnode fields of a word tree.
///
/// Mirrors the C `clear_node` function exactly.
///
/// # Safety
/// `node` must be a valid (or null) wordnode_T pointer from C.
unsafe fn clear_node_inner(node: *mut WordnodeT) {
    if node.is_null() {
        return;
    }
    let mut np = node;
    while !np.is_null() {
        (*np).wn_u1.index = 0;
        (*np).wn_u2.wnode = std::ptr::null_mut();
        let byte = (*np).wn_byte;
        if byte != 0 {
            clear_node_inner((*np).wn_child);
        }
        np = (*np).wn_sibling;
    }
}

/// Write a word tree to `buf` and return the node count (idx returned by traversal).
///
/// Two modes:
///   - Count mode: `buf` is NULL. Traverses the tree, sets `wn_u1.index` and
///     `wn_u2.wnode` fields, returns nodecount, sets `*written_out = 0`.
///   - Write mode: `buf` is non-NULL. Traverses the tree, produces bytes in buf,
///     returns nodecount, sets `*written_out` to bytes produced.
///
/// Caller pattern (matches C `clear_node` + two `put_node` calls):
///   1. `rs_clear_node(tree)`
///   2. `nodecount = rs_put_node(tree, NULL, 0, 0, regionmask, prefixtree, &dummy)`
///   3. Write `nodecount` (4 bytes) to file
///   4. `rs_clear_node(tree)` (reset for write pass)
///   5. `rs_put_node(tree, buf, buf_len, 0, regionmask, prefixtree, &written)`
///   6. Write `buf[0..written]` to file
///
/// Returns nodecount on success, -1 on buffer-too-small error.
///
/// # Safety
/// `node` must be valid (or null) wordnode_T pointer. `buf` must be valid for `buf_len` bytes if non-NULL.
/// `written_out` must be a valid pointer.
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::too_many_arguments)]
pub unsafe extern "C" fn rs_put_node(
    node: *mut WordnodeT,
    buf: *mut u8,
    buf_len: usize,
    idx: c_int,
    regionmask: c_int,
    prefixtree: bool,
    written_out: *mut usize,
) -> c_int {
    let mut out: Vec<u8> = Vec::with_capacity(4096);
    let nodecount = put_node_inner(node, &mut out, idx, regionmask, prefixtree);

    if buf.is_null() {
        // Count-only mode: caller doesn't want bytes, just nodecount.
        if !written_out.is_null() {
            *written_out = 0;
        }
        return nodecount;
    }

    if out.len() > buf_len {
        return -1;
    }

    if !out.is_empty() {
        std::slice::from_raw_parts_mut(buf, out.len()).copy_from_slice(&out);
    }
    if !written_out.is_null() {
        *written_out = out.len();
    }
    nodecount
}

/// Clear the index and wnode fields of a word tree node and its descendants.
///
/// # Safety
/// `node` must be a valid (or null) wordnode_T pointer from C.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_node(node: *mut WordnodeT) {
    clear_node_inner(node);
}

// =============================================================================
// Phase 6: Tree compression (node_compress / wordtree_compress)
// =============================================================================

extern "C" {
    // C callbacks needed by compression.
    fn nvim_deref_wordnode(spin: *mut SpellinfoT, node: *mut WordnodeT) -> c_int;
    #[link_name = "got_int"]
    static got_int_spell: bool;
    fn veryfast_breakcheck();
}

/// Recursively compress a node and its siblings and children (depth-first).
///
/// Returns the number of compressed (deduplicated) nodes.
/// `tot` is incremented with the total number of nodes visited.
///
/// This mirrors the C `node_compress` function, using a Rust `HashMap`
/// instead of Vim's `hashtab_T`.
///
/// # Safety
/// - `spin` must be a valid `spellinfo_T` pointer.
/// - `node` must be a valid (non-null) `wordnode_T` pointer.
#[allow(clippy::cast_sign_loss)]
unsafe fn node_compress_inner(
    spin: *mut SpellinfoT,
    node: *mut WordnodeT,
    ht: &mut std::collections::HashMap<[u8; 6], Vec<*mut WordnodeT>>,
    tot: &mut c_int,
) -> c_int {
    let mut len: c_int = 0;
    let mut compressed: c_int = 0;

    // Walk the sibling list.
    let mut np = node;
    while !np.is_null() {
        if got_int_spell {
            break;
        }
        len += 1;
        let child = (*np).wn_child;
        if !child.is_null() {
            // Recursively compress the child subtree first.
            compressed += node_compress_inner(spin, child, ht, tot);

            // Build the hash key for the (now-compressed) child list.
            let key = (*child).wn_u1.hashkey;

            // Look up the key in the HashMap.
            let bucket = ht.entry(key).or_default();
            let mut found: *mut WordnodeT = std::ptr::null_mut();
            for &tp in bucket.iter() {
                if node_equal_inner(child, tp) {
                    found = tp;
                    break;
                }
            }

            if found.is_null() {
                // No identical child found: add to bucket.
                bucket.push(child);
            } else {
                // Found identical child: replace and free the current one.
                (*found).wn_refs += 1;
                compressed += nvim_deref_wordnode(spin, child);
                (*np).wn_child = found;
            }
        }
        np = (*np).wn_sibling;
    }

    *tot += len + 1; // +1 for length field

    // Build hash key for this sibling list.
    let mut nr: u32 = 0;
    let mut np = node;
    while !np.is_null() {
        let n = if (*np).wn_byte == 0 {
            // NUL byte node: encode flags + region + affixID
            ((*np).wn_flags as u32)
                + (((*np).wn_region as u16 as u32) << 8)
                + (((*np).wn_affixID as u32) << 16)
        } else {
            // byte node: encode byte + child pointer address
            ((*np).wn_byte as u32) + ((((*np).wn_child as usize) as u32) << 8)
        };
        nr = nr.wrapping_mul(101).wrapping_add(n);
        np = (*np).wn_sibling;
    }

    let mut key = [0u8; 6];
    key[0] = len as u8;
    key[1] = {
        let b = (nr & 0xff) as u8;
        if b == 0 {
            1
        } else {
            b
        }
    };
    key[2] = {
        let b = ((nr >> 8) & 0xff) as u8;
        if b == 0 {
            1
        } else {
            b
        }
    };
    key[3] = {
        let b = ((nr >> 16) & 0xff) as u8;
        if b == 0 {
            1
        } else {
            b
        }
    };
    key[4] = {
        let b = ((nr >> 24) & 0xff) as u8;
        if b == 0 {
            1
        } else {
            b
        }
    };
    key[5] = 0; // NUL terminator

    (*node).wn_u1.hashkey = key;

    veryfast_breakcheck();

    compressed
}

/// Check whether two sibling lists are identical (same bytes, flags, children).
///
/// Mirrors the C `node_equal` function.
///
/// # Safety
/// Both `n1` and `n2` must be valid (or null) `wordnode_T` pointers.
unsafe fn node_equal_inner(n1: *mut WordnodeT, n2: *mut WordnodeT) -> bool {
    let mut p1 = n1;
    let mut p2 = n2;
    loop {
        match (p1.is_null(), p2.is_null()) {
            (true, true) => return true,
            (false, false) => {}
            _ => return false,
        }
        let b1 = (*p1).wn_byte;
        let b2 = (*p2).wn_byte;
        if b1 != b2 {
            return false;
        }
        if b1 == 0 {
            // NUL node: compare flags, region, affixID
            if (*p1).wn_flags != (*p2).wn_flags
                || (*p1).wn_region != (*p2).wn_region
                || (*p1).wn_affixID != (*p2).wn_affixID
            {
                return false;
            }
        } else {
            // byte node: compare child pointers (structural sharing)
            if (*p1).wn_child != (*p2).wn_child {
                return false;
            }
        }
        p1 = (*p1).wn_sibling;
        p2 = (*p2).wn_sibling;
    }
}

// (Compression no longer needs a separate child setter; direct field access is used.)

/// Compress the word tree rooted at `root` by deduplicating identical subtrees.
///
/// This is the Rust replacement for the C `node_compress` function.
/// The caller (`wordtree_compress` in C) handles verbose message display.
///
/// Returns `(compressed_count, total_count)`.
///
/// # Safety
/// - `spin` must be a valid `spellinfo_T` pointer.
/// - `root` must be the first sibling (i.e., `root->wn_sibling` of the tree root).
#[no_mangle]
pub unsafe extern "C" fn rs_node_compress(
    spin: *mut SpellinfoT,
    node: *mut WordnodeT,
    tot_out: *mut c_int,
) -> c_int {
    if node.is_null() {
        return 0;
    }
    let mut ht = std::collections::HashMap::new();
    let mut tot: c_int = 0;
    let compressed = node_compress_inner(spin, node, &mut ht, &mut tot);
    if !tot_out.is_null() {
        *tot_out = tot;
    }
    compressed
}

// =============================================================================
// Phase 5: tree_add_word, store_word
// =============================================================================

extern "C" {
    fn getroom(spin: *mut SpellinfoT, len: usize, align: bool) -> *mut c_void;
    // Message functions for compression progress output
    fn msg_start();
    fn msg_puts(s: *const c_char);
    fn msg_clr_eos();
    fn ui_flush();
    fn msg(s: *const c_char, hl_id: c_int) -> bool;
    fn verbose_enter();
    fn verbose_leave();
    // curwin global for spell_casefold
    #[link_name = "curwin"]
    static curwin_spell: *mut c_void;
    // msg_didout and msg_col are writable globals
    static mut msg_didout: bool;
    static mut msg_col: c_int;
    static p_verbose: i64;
}

// =============================================================================
// Phase 2: wordnode / wordtree helpers exported to C
// =============================================================================

/// Allocate and return a fresh WordnodeT from spin's arena.
///
/// Reuses nodes from the free list (si_first_free) before allocating from
/// the arena.  Returns null on allocation failure.
///
/// # Safety
/// `spin` must be a valid non-null pointer.
#[export_name = "get_wordnode"]
pub unsafe extern "C" fn rs_get_wordnode_export(spin: *mut SpellinfoT) -> *mut WordnodeT {
    get_wordnode(spin)
}

unsafe fn get_wordnode(spin: *mut SpellinfoT) -> *mut WordnodeT {
    let first_free = (*spin).si_first_free;
    if first_free.is_null() {
        getroom(spin, std::mem::size_of::<WordnodeT>(), true).cast::<WordnodeT>()
    } else {
        let n = first_free;
        (*spin).si_first_free = (*n).wn_child;
        std::ptr::write_bytes(n, 0u8, 1);
        (*spin).si_free_count -= 1;
        n
    }
}

/// Allocate a fresh WordnodeT from the arena without touching the free list.
///
/// # Safety
/// `spin` must be a valid non-null pointer.
#[export_name = "wordtree_alloc"]
pub unsafe extern "C" fn rs_wordtree_alloc(spin: *mut SpellinfoT) -> *mut WordnodeT {
    getroom(spin, std::mem::size_of::<WordnodeT>(), true).cast::<WordnodeT>()
}

/// Print a spell-building message when verbose or si_verbose is set.
///
/// # Safety
/// `spin` and `str_` must be valid non-null pointers. `str_` must be a valid
/// NUL-terminated C string.
#[export_name = "spell_message"]
pub unsafe extern "C" fn rs_spell_message(spin: *const SpellinfoT, str_: *const c_char) {
    if (*spin).si_verbose != 0 || p_verbose > 2 {
        if (*spin).si_verbose == 0 {
            verbose_enter();
        }
        msg(str_, 0);
        ui_flush();
        if (*spin).si_verbose == 0 {
            verbose_leave();
        }
    }
}

/// Compress the word tree rooted at `root` and print a progress message.
///
/// # Safety
/// All pointers must be valid. `name` must be a valid NUL-terminated C string.
#[export_name = "wordtree_compress"]
pub unsafe extern "C" fn rs_wordtree_compress_export(
    spin: *mut SpellinfoT,
    root: *mut WordnodeT,
    name: *const c_char,
) {
    let sibling = (*root).wn_sibling;
    if sibling.is_null() {
        return;
    }
    let mut tot: c_int = 0;
    let n = rs_node_compress(spin, sibling, &raw mut tot);
    if (*spin).si_verbose != 0 || p_verbose > 2 {
        let perc: i64 = if tot > 1_000_000 {
            (tot - n) as i64 / (tot as i64 / 100)
        } else if tot == 0 {
            0
        } else {
            (tot - n) as i64 * 100 / tot as i64
        };
        let name_str =
            String::from_utf8_lossy(std::ffi::CStr::from_ptr(name).to_bytes()).into_owned();
        let msg_text = format!(
            "Compressed {name_str}: {n} of {tot} nodes; {} ({}%) remaining\0",
            tot - n,
            perc
        );
        rs_spell_message(spin, msg_text.as_ptr().cast::<c_char>());
    }
}

/// Tracking where to link a new node when inserting.
///
/// Because `wordnode_T` sibling/child fields are accessed via accessor functions,
/// we can't hold `*mut *mut WordnodeT` into struct fields. Instead we track which
/// node owns the pointer-slot and whether it's the sibling or child slot.
#[derive(Copy, Clone)]
enum Prev {
    /// No parent yet (only happens when `node = root` before advancing).
    None,
    /// Link via the `wn_sibling` field of this node.
    Sibling(*mut WordnodeT),
    /// Link via the `wn_child` field of this node.
    Child(*mut WordnodeT),
}

impl Prev {
    /// Write `val` into the tracked field.
    #[allow(clippy::enum_variant_names)]
    unsafe fn set(self, val: *mut WordnodeT) {
        match self {
            Self::None => {}
            Self::Sibling(p) => (*p).wn_sibling = val,
            Self::Child(p) => (*p).wn_child = val,
        }
    }
}

/// Rust implementation of `tree_add_word`.
///
/// Adds a NUL-terminated `word` to the trie rooted at `root`.
/// Returns 0 (OK) or 1 (FAIL).
///
/// # Safety
/// All pointers must be valid. `word` must be a valid NUL-terminated C string.
#[no_mangle]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_tree_add_word(
    spin: *mut SpellinfoT,
    word: *const c_char,
    root: *mut WordnodeT,
    flags: c_int,
    region: c_int,
    affix_id: c_int,
) -> c_int {
    let word_len = libc::strlen(word) as usize;
    // Build a slice including the NUL terminator for iteration
    let word_bytes = std::slice::from_raw_parts(word.cast::<u8>(), word_len + 1);

    let mut node = root;
    let mut prev = Prev::None;

    for &byte_i in word_bytes {
        // ---- COW: if this node has more than one reference, copy the sibling list ----
        if !node.is_null() && (*node).wn_refs > 1 {
            (*node).wn_refs -= 1;
            let mut copy_prev = prev;
            let original_node = node;
            let mut copy_p = original_node;
            let mut first_copy: *mut WordnodeT = std::ptr::null_mut();
            while !copy_p.is_null() {
                let np = get_wordnode(spin);
                if np.is_null() {
                    return 1; // FAIL
                }
                // Copy all fields
                let child = (*copy_p).wn_child;
                (*np).wn_child = child;
                if !child.is_null() {
                    (*child).wn_refs += 1;
                }
                let b = (*copy_p).wn_byte;
                (*np).wn_byte = b;
                if b == 0 {
                    (*np).wn_flags = (*copy_p).wn_flags;
                    (*np).wn_region = (*copy_p).wn_region;
                    (*np).wn_affixID = (*copy_p).wn_affixID;
                }
                (*np).wn_refs = 1;
                // Link this copy into the new chain via copy_prev
                copy_prev.set(np);
                copy_prev = Prev::Sibling(np);
                // Track the first copy so we can update `node`
                if copy_p == original_node {
                    first_copy = np;
                }
                copy_p = (*copy_p).wn_sibling;
            }
            // `node` now points to the first copy
            node = first_copy;
        }

        // ---- Find the sibling with the matching byte ----
        while !node.is_null() {
            let nb = (*node).wn_byte;
            let advance = if byte_i == 0 {
                // NUL node: compare on flags/region/affixID
                if flags < 0 {
                    ((*node).wn_affixID as c_int) < affix_id
                } else if ((*node).wn_flags as c_int) < (flags & WN_MASK) {
                    true
                } else if ((*node).wn_flags as c_int) == (flags & WN_MASK) {
                    if (*spin).si_sugtree != 0 {
                        (((*node).wn_region as c_int) & 0xffff) < region
                    } else {
                        ((*node).wn_affixID as c_int) < affix_id
                    }
                } else {
                    false
                }
            } else {
                nb < byte_i
            };
            if !advance {
                break;
            }
            prev = Prev::Sibling(node);
            node = (*node).wn_sibling;
        }

        // ---- Check if the current node matches this byte ----
        let matched = if node.is_null() {
            false
        } else {
            let nb = (*node).wn_byte;
            if nb != byte_i {
                false
            } else if byte_i == 0 {
                // NUL: match only if flags/affixID also match (and not sugtree)
                flags >= 0
                    && (*spin).si_sugtree == 0
                    && ((*node).wn_flags as c_int) == (flags & WN_MASK)
                    && ((*node).wn_affixID as c_int) == affix_id
            } else {
                true
            }
        };

        if !matched {
            // ---- Allocate a new node ----
            let np = get_wordnode(spin);
            if np.is_null() {
                return 1; // FAIL
            }
            (*np).wn_byte = byte_i;
            if node.is_null() {
                (*np).wn_refs = 1;
            } else {
                (*np).wn_refs = (*node).wn_refs;
                (*node).wn_refs = 1;
            }
            prev.set(np);
            (*np).wn_sibling = node;
            node = np;
        }

        // ---- At end of word, set flags/region/affixID and stop ----
        if byte_i == 0 {
            (*node).wn_flags = (flags & WN_MASK) as u16;
            (*node).wn_region |= region as i16;
            (*node).wn_affixID = affix_id as u8;
            break;
        }

        // ---- Go deeper ----
        prev = Prev::Child(node);
        node = (*node).wn_child;
    }

    // ---- Increment message counter ----
    (*spin).si_msg_count += 1;

    // ---- Compression trigger logic ----
    let compress_cnt = (*spin).si_compress_cnt;
    if compress_cnt > 1 {
        let new_cnt = compress_cnt - 1;
        (*spin).si_compress_cnt = new_cnt;
        if new_cnt == 1 {
            (*spin).si_blocks_cnt += COMPRESS_INC;
        }
    }

    // Check if compression is needed
    #[allow(clippy::cast_possible_wrap)]
    let should_compress = if compress_cnt == 1 {
        (*spin).si_free_count < MAXWLEN as c_int
    } else {
        (*spin).si_blocks_cnt >= COMPRESS_START
    };

    if should_compress {
        (*spin).si_blocks_cnt -= COMPRESS_INC;
        (*spin).si_compress_cnt = COMPRESS_ADDED;
        // Show compression message if verbose
        if (*spin).si_verbose != 0 {
            msg_start();
            msg_puts(c"Compressing word tree...".as_ptr());
            msg_clr_eos();
            msg_didout = false;
            msg_col = 0;
            ui_flush();
        }
        rs_wordtree_compress_export(spin, (*spin).si_foldroot, c"case-folded".as_ptr());
        if affix_id >= 0 {
            rs_wordtree_compress_export(spin, (*spin).si_keeproot, c"keep-case".as_ptr());
        }
    }

    0 // OK
}

/// Word flags mask (matches C `WN_MASK = 0xffff`).
const WN_MASK: c_int = 0xffff;

/// WF_KEEPCAP flag value (matches spell_defs.h).
const WF_KEEPCAP: c_int = 0x80;

/// `MAXWLEN + 1` (255), for `nvim_spell_casefold` buffer length.
const FOLDWORD_LEN: c_int = 255;

/// Rust implementation of `store_word`.
///
/// Stores a word in the case-folded tree (and keep-case tree if KEEPCAP).
/// Returns 0 (OK) or 1 (FAIL).
///
/// # Safety
/// - `spin` must be a valid spellinfo_T pointer.
/// - `word` must be a valid NUL-terminated C string.
/// - `pfxlist` must be NULL or a valid NUL-terminated byte string of affix IDs.
#[no_mangle]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn rs_store_word(
    spin: *mut SpellinfoT,
    word: *const c_char,
    flags: c_int,
    region: c_int,
    pfxlist: *const c_char,
    need_affix: bool,
) -> c_int {
    let word_len = libc::strlen(word) as c_int;
    let end = word.add(word_len as usize);

    // Reject words with illegal characters.
    if !crate::rs_valid_spell_word(word.cast::<u8>(), end.cast::<u8>()) {
        return 1; // FAIL
    }

    let ct = crate::rs_captype(word, end);

    // Case-fold the word.
    let mut foldword = [0i8; MAXWLEN + 1];
    crate::check::rs_spell_casefold_c_compat(
        curwin_spell.cast::<c_void>(),
        word,
        word_len,
        foldword.as_mut_ptr(),
        FOLDWORD_LEN,
    );

    let mut res = 0i32;
    let foldroot = (*spin).si_foldroot;

    // Store in case-folded tree for each prefix/compound ID in pfxlist.
    {
        let mut p: *const c_char = if pfxlist.is_null() {
            std::ptr::null()
        } else {
            pfxlist
        };
        loop {
            if !need_affix || (!p.is_null() && *p != 0) {
                let affix_id = if p.is_null() { 0 } else { (*p as u8) as c_int };
                let r = rs_tree_add_word(
                    spin,
                    foldword.as_ptr(),
                    foldroot,
                    ct | flags,
                    region,
                    affix_id,
                );
                if r != 0 {
                    res = r;
                }
            }
            if p.is_null() || *p == 0 {
                break;
            }
            p = p.add(1);
        }
    }
    (*spin).si_foldwcount += 1;

    // Also store in keep-case tree if the word is flagged as keep-case.
    if res == 0 && (ct == WF_KEEPCAP || (flags & WF_KEEPCAP) != 0) {
        let keeproot = (*spin).si_keeproot;
        let mut p: *const c_char = if pfxlist.is_null() {
            std::ptr::null()
        } else {
            pfxlist
        };
        loop {
            if !need_affix || (!p.is_null() && *p != 0) {
                let affix_id = if p.is_null() { 0 } else { (*p as u8) as c_int };
                let r = rs_tree_add_word(spin, word, keeproot, flags, region, affix_id);
                if r != 0 {
                    res = r;
                }
            }
            if p.is_null() || *p == 0 {
                break;
            }
            p = p.add(1);
        }
        (*spin).si_keepwcount += 1;
    }

    res
}

// =============================================================================
// Phase 5b: deref_wordnode and free_wordnode
// =============================================================================

// (nvim_spin_get/set_first_free and nvim_spin_set_free_count removed in Phase 2;
// using direct SpellinfoT field access now.)

/// Rust replacement for C `deref_wordnode` / `nvim_deref_wordnode`.
///
/// Decrement the reference count on a node (head of a sibling list). If it
/// reaches zero, recursively free all children then all siblings, returning
/// the number of nodes freed (plus one for the length field).
///
/// # Safety
/// `spin` and `node` must be valid non-null pointers.
#[export_name = "nvim_deref_wordnode"]
pub unsafe extern "C" fn rs_deref_wordnode(spin: *mut SpellinfoT, node: *mut WordnodeT) -> c_int {
    (*node).wn_refs -= 1;
    if (*node).wn_refs != 0 {
        return 0;
    }

    let mut cnt = 0;
    let mut np = node;
    while !np.is_null() {
        let child = (*np).wn_child;
        if !child.is_null() {
            cnt += rs_deref_wordnode(spin, child);
        }
        let sibling = (*np).wn_sibling;
        rs_free_wordnode(spin, np);
        cnt += 1;
        np = sibling;
    }
    cnt + 1 // length field
}

/// Rust replacement for C `free_wordnode`.
///
/// Returns a node to the free list in `spin->si_first_free`.
///
/// # Safety
/// `spin` and `n` must be valid non-null pointers.
unsafe fn rs_free_wordnode(spin: *mut SpellinfoT, n: *mut WordnodeT) {
    // Chain this node onto the free list via wn_child.
    (*n).wn_child = (*spin).si_first_free;
    (*spin).si_first_free = n;
    (*spin).si_free_count += 1;
}

// =============================================================================
// Phase 1: Pure affix utility functions (no Vim API dependencies)
// =============================================================================

/// Returns true when items[0] equals "rulename", there are "mincount" items or
/// a comment follows after item "mincount".
///
/// # Safety
/// - `items` must be a valid pointer to at least `itemcnt` valid C string pointers.
/// - `rulename` must be a valid NUL-terminated C string.
#[export_name = "is_aff_rule"]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_is_aff_rule(
    items: *const *const c_char,
    itemcnt: c_int,
    rulename: *const c_char,
    mincount: c_int,
) -> bool {
    if items.is_null() || rulename.is_null() || itemcnt < 1 {
        return false;
    }
    let item0 = *items;
    if item0.is_null() {
        return false;
    }
    // Compare items[0] with rulename
    let a = std::ffi::CStr::from_ptr(item0);
    let b = std::ffi::CStr::from_ptr(rulename);
    if a != b {
        return false;
    }
    if itemcnt == mincount {
        return true;
    }
    if itemcnt > mincount && mincount >= 0 {
        let item_at_min = *items.add(mincount as usize);
        // c_char is i8; '#' = 0x23, no sign-extension issue
        if !item_at_min.is_null() && (*item_at_min as u8) == b'#' {
            return true;
        }
    }
    false
}

/// Returns true if "s" is the name of an info item in the affix file.
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string.
#[export_name = "spell_info_item"]
pub unsafe extern "C" fn rs_spell_info_item(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }
    let cs = std::ffi::CStr::from_ptr(s);
    matches!(
        cs.to_bytes(),
        b"NAME" | b"HOME" | b"VERSION" | b"AUTHOR" | b"EMAIL" | b"COPYRIGHT"
    )
}

/// Returns true if strings "s1" and "s2" are equal.  Also considers both being
/// NULL as equal.
///
/// # Safety
/// - If non-null, `s1` and `s2` must be valid NUL-terminated C strings.
#[export_name = "str_equal"]
pub unsafe extern "C" fn rs_str_equal(s1: *const c_char, s2: *const c_char) -> bool {
    match (s1.is_null(), s2.is_null()) {
        (true, true) => true,
        (false, false) => {
            let a = std::ffi::CStr::from_ptr(s1);
            let b = std::ffi::CStr::from_ptr(s2);
            a == b
        }
        _ => false,
    }
}

/// qsort comparator for REP items (fromto_T): compares ft_from strings.
///
/// # Safety
/// - `s1` and `s2` must point to valid `fromto_T` structs with non-null `ft_from`.
#[export_name = "rep_compare"]
pub unsafe extern "C" fn rs_rep_compare(s1: *const c_void, s2: *const c_void) -> c_int {
    // fromto_T layout: ft_from (*char) then ft_to (*char)
    // We only need to compare ft_from, which is the first field.
    let from1 = *(s1 as *const *const c_char);
    let from2 = *(s2 as *const *const c_char);
    if from1.is_null() && from2.is_null() {
        return 0;
    }
    if from1.is_null() {
        return -1;
    }
    if from2.is_null() {
        return 1;
    }
    libc::strcmp(from1, from2)
}

// =============================================================================
// Phase 3: sugfile generation (spell_make_sugfile and helpers)
// =============================================================================

extern "C" {
    #[link_name = "line_breakcheck"]
    fn line_breakcheck_sug();
    #[link_name = "ga_init"]
    fn ga_init_sug(gap: *mut crate::GArrayRaw, itemsize: c_int, growsize: c_int);
    #[link_name = "ga_clear"]
    fn ga_clear_sug(gap: *mut crate::GArrayRaw);
    fn ml_append_buf(
        buf: *mut c_void,
        lnum: i32,
        line: *mut c_char,
        len: i32,
        newfile: bool,
    ) -> c_int;
    fn ml_get_buf(buf: *mut c_void, lnum: i32) -> *mut c_char;
    fn ml_get_buf_len(buf: *mut c_void, lnum: i32) -> i32;
    fn open_spellbuf() -> *mut c_void;
    fn close_spellbuf(buf: *mut c_void);
    #[link_name = "os_fopen"]
    fn os_fopen_sug(fname: *const c_char, mode: *const c_char) -> *mut libc::FILE;
    fn put_bytes(fd: *mut libc::FILE, number: u64, len: usize) -> bool;
    fn put_time(fd: *mut libc::FILE, time_: i64) -> c_int;
    fn free_blocks(bl: *mut c_void);
    fn slang_free(lp: *mut crate::SlangRaw);
    #[link_name = "path_full_compare"]
    fn path_full_compare_sug(
        s1: *mut c_char,
        s2: *mut c_char,
        checkname: bool,
        expandenv: bool,
    ) -> c_int;
    fn nvim_decor_buf_get_line_count(buf_ptr: *mut c_void) -> c_int;
    #[link_name = "smsg"]
    fn smsg_sug(hl_id: c_int, fmt: *const c_char, ...) -> c_int;
    #[link_name = "semsg"]
    fn semsg_sug(fmt: *const c_char, ...) -> bool;
    #[link_name = "first_lang"]
    static first_lang_sug: *mut crate::SlangRaw;
    #[link_name = "e_write"]
    static e_write_sug: *const c_char;
    #[link_name = "e_notopen"]
    static e_notopen_sug: *const c_char;
}

// Declarations for spell_load_file migration (Phase 1)
extern "C" {
    fn slang_alloc(lang: *mut c_char) -> *mut crate::SlangRaw;
    fn slang_clear(lp: *mut crate::SlangRaw);
    #[allow(dead_code)] // used in Phase 2 (suggest_load_files)
    fn slang_clear_sug(lp: *mut crate::SlangRaw);
    fn read_string(fd: *mut libc::FILE, cnt: usize) -> *mut c_char;
    fn get4c(fd: *mut libc::FILE) -> c_int;
    fn get8ctime(fd: *mut libc::FILE) -> i64;
    // ETYPE_SPELL = 9 (index of ETYPE_SPELL in etype_T enum, 0-based)
    fn estack_push(etype: c_int, name: *const c_char, lnum: i32);
    fn estack_pop();
    fn path_tail(fname: *const c_char) -> *mut c_char;
    #[link_name = "xstrdup"]
    fn xstrdup_slang(s: *const c_char) -> *mut c_char;
    #[link_name = "path_full_compare"]
    fn path_full_compare_slang(
        s1: *mut c_char,
        s2: *mut c_char,
        checkname: bool,
        expandenv: bool,
    ) -> c_int;
    fn redraw_all_later(type_: c_int);
    #[link_name = "parse_spelllang"]
    fn parse_spelllang_slang(win: *mut c_void) -> *mut c_char;
    fn path_fnamecmp(p1: *const c_char, p2: *const c_char) -> c_int;
    fn nvim_win_get_b_langp(wp: *const c_void) -> *const crate::GArrayRaw;
    // Error message strings
    #[link_name = "e_format"]
    static e_format_slang: *const c_char;
    #[link_name = "e_notopen"]
    static e_notopen_slang: *const c_char;
    // first_lang as mutable for list insertion
    #[link_name = "first_lang"]
    static mut first_lang_mut: *mut crate::SlangRaw;
    // curwin
    #[link_name = "curwin"]
    static curwin_slang: *mut c_void;
    // p_verbose
    #[link_name = "p_verbose"]
    static p_verbose_slang: i64;
    // semsg / smsg / emsg / verbose_enter / verbose_leave
    #[link_name = "semsg"]
    fn semsg_slang(fmt: *const c_char, ...) -> bool;
    #[link_name = "smsg"]
    fn smsg_slang(hl_id: c_int, fmt: *const c_char, ...) -> c_int;
    #[link_name = "emsg"]
    fn emsg_slang(s: *const c_char) -> bool;
    #[link_name = "verbose_enter"]
    fn verbose_enter_slang();
    #[link_name = "verbose_leave"]
    fn verbose_leave_slang();
    // xfree / xmalloc / xcalloc
    #[link_name = "xfree"]
    fn xfree_slang(ptr: *mut c_void);
    #[link_name = "xmalloc"]
    fn xmalloc_slang(size: usize) -> *mut c_void;
    #[link_name = "xcalloc"]
    fn xcalloc_slang(count: usize, size: usize) -> *mut c_void;
    // os_fopen
    #[link_name = "os_fopen"]
    fn os_fopen_slang(fname: *const c_char, mode: *const c_char) -> *mut libc::FILE;
}

/// Magic bytes and version for .sug file header.
const VIMSUGMAGIC: &[u8] = b"VIMsug";
const VIMSUGVERSION: u8 = 1;
/// kEqualFiles: path_full_compare return value when both paths refer to the same file.
const K_EQUAL_FILES: c_int = 1;

/// Build the soundfold trie for language `slang`.
///
/// Traverses `sl_fbyts`/`sl_fidxs`, soundfolds each word, and inserts it into
/// `spin->si_foldroot` via `rs_tree_add_word`.
///
/// Returns 0 (OK) or 1 (FAIL).
///
/// # Safety
/// `spin` and `slang` must be valid non-null pointers.
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
unsafe fn sug_filltree(spin: *mut SpellinfoT, slang: *mut crate::SlangRaw) -> c_int {
    const MAXWLEN: usize = 254;
    const OK: c_int = 0;
    const FAIL: c_int = 1;

    let byts = (*slang).sl_fbyts;
    let idxs = (*slang).sl_fidxs;
    if byts.is_null() || idxs.is_null() {
        return FAIL;
    }
    let fbyts_len = (*slang).sl_fbyts_len as usize;

    (*spin).si_foldroot = rs_wordtree_alloc(spin);
    (*spin).si_sugtree = 1;

    let mut arridx = [0i32; MAXWLEN];
    let mut curi = [0i32; MAXWLEN];
    let mut wordcount = [0i32; MAXWLEN];
    let mut tword = [0u8; MAXWLEN];
    let mut tsalword = [0u8; MAXWLEN + 4];

    arridx[0] = 0;
    curi[0] = 1;
    wordcount[0] = 0;

    let mut depth: i32 = 0;
    let mut words_done: u32 = 0;

    while depth >= 0 && !got_int_spell {
        let d = depth as usize;
        let cur_arr = arridx[d] as usize;
        let n_siblings = *byts.add(cur_arr) as i32;
        if curi[d] > n_siblings {
            *idxs.add(cur_arr) = wordcount[d];
            if depth > 0 {
                wordcount[d - 1] += wordcount[d];
            }
            depth -= 1;
            line_breakcheck_sug();
        } else {
            let n = cur_arr + curi[d] as usize;
            curi[d] += 1;

            let c = *byts.add(n);
            if c == 0 {
                tword[d] = 0;
                crate::rs_spell_soundfold(
                    slang,
                    tword.as_mut_ptr().cast::<c_char>(),
                    true,
                    tsalword.as_mut_ptr().cast::<c_char>(),
                );

                let flags = (words_done >> 16) as c_int;
                let region = (words_done & 0xffff) as c_int;
                if rs_tree_add_word(
                    spin,
                    tsalword.as_ptr().cast::<c_char>(),
                    (*spin).si_foldroot,
                    flags,
                    region,
                    0,
                ) == FAIL
                {
                    return FAIL;
                }

                words_done += 1;
                wordcount[d] += 1;
                (*spin).si_blocks_cnt = 0;

                let mut nn = n;
                while nn + 1 < fbyts_len && *byts.add(nn + 1) == 0 {
                    nn += 1;
                    curi[d] += 1;
                }
            } else {
                tword[d] = c;
                depth += 1;
                let nd = depth as usize;
                arridx[nd] = *idxs.add(n);
                curi[nd] = 1;
                wordcount[nd] = 0;
            }
        }
    }

    smsg_sug(
        0,
        c"Total number of words: %d".as_ptr(),
        words_done as c_int,
    );

    OK
}

/// Fill the suggestion table for one node and its children.
///
/// Returns the next word number, or -1 on OOM.
///
/// # Safety
/// All pointers must be valid.
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
unsafe fn sug_filltable(
    spin: *mut SpellinfoT,
    node: *mut WordnodeT,
    startwordnr: c_int,
    gap: *mut crate::GArrayRaw,
) -> c_int {
    const FAIL: c_int = -1;
    let mut wordnr = startwordnr;

    let mut p = node;
    while !p.is_null() {
        if (*p).wn_byte == 0 {
            (*gap).ga_len = 0;
            let mut prev_nr: i32 = 0;

            let mut np = p;
            while !np.is_null() && (*np).wn_byte == 0 {
                ga_grow(gap, 10);

                let nr = (((*np).wn_flags as i32) << 16) + ((*np).wn_region as i32 & 0xffff);
                let delta = nr - prev_nr;
                prev_nr += delta;

                let ga_data = (*gap).ga_data.cast::<u8>();
                let ga_len = (*gap).ga_len as usize;
                let buf = std::slice::from_raw_parts_mut(ga_data.add(ga_len), 10);
                let written = crate::offset2bytes(delta, buf);
                (*gap).ga_len += written as c_int;

                np = (*np).wn_sibling;
            }

            let ga_data = (*gap).ga_data.cast::<u8>();
            *ga_data.add((*gap).ga_len as usize) = 0u8;
            (*gap).ga_len += 1;

            if ml_append_buf(
                (*spin).si_spellbuf,
                wordnr,
                (*gap).ga_data.cast::<c_char>(),
                (*gap).ga_len,
                true,
            ) != 0
            {
                return FAIL;
            }
            wordnr += 1;

            while !(*p).wn_sibling.is_null() && (*(*p).wn_sibling).wn_byte == 0 {
                (*p).wn_sibling = (*(*p).wn_sibling).wn_sibling;
            }

            (*p).wn_flags = 0;
            (*p).wn_region = 0;
        } else {
            wordnr = sug_filltable(spin, (*p).wn_child, wordnr, gap);
            if wordnr == FAIL {
                return FAIL;
            }
        }
        p = (*p).wn_sibling;
    }
    wordnr
}

/// Create the memline table linking soundfold words to source words.
///
/// # Safety
/// `spin` must be a valid non-null pointer.
unsafe fn sug_maketable(spin: *mut SpellinfoT) -> c_int {
    const OK: c_int = 0;
    const FAIL: c_int = 1;

    (*spin).si_spellbuf = open_spellbuf();

    let mut ga = crate::GArrayRaw {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: std::ptr::null_mut(),
    };
    ga_init_sug(&raw mut ga, 1, 100);

    let sibling = (*(*spin).si_foldroot).wn_sibling;
    let res = if sug_filltable(spin, sibling, 0, &raw mut ga) == -1 {
        FAIL
    } else {
        OK
    };

    ga_clear_sug(&raw mut ga);
    res
}

/// Write the .sug file to `fname`.
///
/// # Safety
/// `spin` and `fname` must be valid non-null pointers.
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation
)]
unsafe fn sug_write(spin: *mut SpellinfoT, fname: *const c_char) {
    let fd = os_fopen_sug(fname, c"w".as_ptr());
    if fd.is_null() {
        semsg_sug(e_notopen_sug, fname);
        return;
    }

    {
        let fname_str = std::ffi::CStr::from_ptr(fname).to_string_lossy();
        let msg_text = format!("Writing suggestion file {fname_str}...\0");
        rs_spell_message(spin, msg_text.as_ptr().cast::<c_char>());
    }

    // <SUGHEADER>: <fileID> <versionnr> <timestamp>
    if libc::fwrite(
        VIMSUGMAGIC.as_ptr().cast::<c_void>(),
        VIMSUGMAGIC.len(),
        1,
        fd,
    ) != 1
    {
        emsg(e_write_sug);
        libc::fclose(fd);
        return;
    }
    libc::fputc(c_int::from(VIMSUGVERSION), fd);
    put_time(fd, (*spin).si_sugtime);

    // <SUGWORDTREE>
    (*spin).si_memtot = 0;
    let tree = (*(*spin).si_foldroot).wn_sibling;

    rs_clear_node(tree);
    let mut sug_dummy: usize = 0;
    let sug_nodecount = rs_put_node(
        tree,
        std::ptr::null_mut(),
        0,
        0,
        0,
        false,
        &raw mut sug_dummy,
    );
    if sug_nodecount < 0 {
        emsg(e_write_sug);
        libc::fclose(fd);
        return;
    }

    put_bytes(fd, sug_nodecount as u64, 4);
    (*spin).si_memtot += sug_nodecount + sug_nodecount * std::mem::size_of::<c_int>() as c_int;

    let sug_tree_buf_len = (sug_nodecount as usize) * 8 + 1024;
    let sug_tree_buf = libc::malloc(sug_tree_buf_len).cast::<u8>();
    if sug_tree_buf.is_null() {
        emsg(e_write_sug);
        libc::fclose(fd);
        return;
    }
    rs_clear_node(tree);
    let mut sug_tree_written: usize = 0;
    let sug_nc2 = rs_put_node(
        tree,
        sug_tree_buf,
        sug_tree_buf_len,
        0,
        0,
        false,
        &raw mut sug_tree_written,
    );
    if sug_nc2 < 0
        || (sug_tree_written > 0
            && libc::fwrite(sug_tree_buf.cast::<c_void>(), sug_tree_written, 1, fd) != 1)
    {
        libc::free(sug_tree_buf.cast::<c_void>());
        emsg(e_write_sug);
        libc::fclose(fd);
        return;
    }
    libc::free(sug_tree_buf.cast::<c_void>());

    // <SUGTABLE>: <sugwcount> <sugline> ...
    let wcount = nvim_decor_buf_get_line_count((*spin).si_spellbuf);
    put_bytes(fd, wcount as u64, 4);

    let mut write_ok = true;
    for lnum in 1..=wcount {
        let line = ml_get_buf((*spin).si_spellbuf, lnum);
        let len = ml_get_buf_len((*spin).si_spellbuf, lnum) + 1;
        if libc::fwrite(line.cast::<c_void>(), len as usize, 1, fd) == 0 {
            emsg(e_write_sug);
            write_ok = false;
            break;
        }
        (*spin).si_memtot += len;
    }

    if write_ok {
        if libc::fputc(0, fd) == libc::EOF {
            emsg(e_write_sug);
        } else {
            let msg_text = format!(
                "Estimated runtime memory use: {} bytes\0",
                (*spin).si_memtot
            );
            rs_spell_message(spin, msg_text.as_ptr().cast::<c_char>());
        }
    }

    libc::fclose(fd);
}

// =============================================================================
// Phase: spell_load_file migration
// =============================================================================

/// ETYPE_SPELL value: position of ETYPE_SPELL in the etype_T enum (0-indexed).
const ETYPE_SPELL_VAL: c_int = 9;

/// SPL_FNAME_ADD: suffix that marks add-word spell files.
const SPL_FNAME_ADD: &[u8] = b".add.\0";

/// UPD_SOME_VALID: redraw type for redraw_all_later.
const UPD_SOME_VALID: c_int = 35;

/// kEqualFiles: path_full_compare return value when both paths point to the same file.
const K_EQUAL_FILES_SLANG: c_int = 1;

/// Load a spell file (.spl) and return the slang_T it was loaded into.
///
/// This is the Rust implementation replacing the C `spell_load_file`.
///
/// Three calling modes:
/// - `lang != NULL`, `old_lp == NULL`: first load, allocates a new slang_T.
/// - `lang == NULL`, `old_lp != NULL`: reload an existing slang_T.
/// - `lang == NULL`, `old_lp == NULL`: read-back after writing (for .sug creation).
///
/// # Safety
/// `fname` must be a valid NUL-terminated string. `lang` and `old_lp` may be null.
#[export_name = "spell_load_file"]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::too_many_lines,
    clippy::if_not_else,
    unused_assignments
)]
pub unsafe extern "C" fn rs_spell_load_file(
    fname: *mut c_char,
    lang: *mut c_char,
    old_lp: *mut crate::SlangRaw,
    silent: bool,
) -> *mut crate::SlangRaw {
    // SP_* error codes
    const SP_FORMERROR: c_int = -2;
    const SP_TRUNCERROR: c_int = -1;
    const SP_OTHERERROR: c_int = -3;
    const OK: c_int = 0;
    const NUL: u8 = 0;
    const EOF: c_int = -1;

    // VIMSPELLMAGIC / VIMSPELLMAGICL
    const VIMSPELLMAGIC_BYTES: &[u8; 8] = b"VIMspell";
    const VIMSPELLMAGICL: usize = 8;
    const VIMSPELLVERSION: c_int = 50;

    // Section IDs (keep in sync with spellfile.rs constants and C enum)
    const SN_REGION_V: u8 = 0;
    const SN_CHARFLAGS_V: u8 = 1;
    const SN_MIDWORD_V: u8 = 2;
    const SN_PREFCOND_V: u8 = 3;
    const SN_REP_V: u8 = 4;
    const SN_SAL_V: u8 = 5;
    const SN_SOFO_V: u8 = 6;
    const SN_MAP_V: u8 = 7;
    const SN_COMPOUND_V: u8 = 8;
    const SN_SYLLABLE_V: u8 = 9;
    const SN_NOBREAK_V: u8 = 10;
    const SN_SUGFILE_V: u8 = 11;
    const SN_REPSAL_V: u8 = 12;
    const SN_WORDS_V: u8 = 13;
    const SN_NOSPLITSUGS_V: u8 = 14;
    const SN_INFO_V: u8 = 15;
    const SN_NOCOMPOUNDSUGS_V: u8 = 16;
    const SN_END_V: u8 = 255;
    const SNF_REQUIRED_V: u8 = 1;

    const MAXWLEN_V: usize = 254;
    const MAXREGIONS: usize = 8;

    let fd = os_fopen_slang(fname, c"r".as_ptr());
    if fd.is_null() {
        if !silent {
            semsg_slang(e_notopen_slang, fname);
        } else if p_verbose_slang > 2 {
            verbose_enter_slang();
            smsg_slang(0, e_notopen_slang, fname);
            verbose_leave_slang();
        }
        return std::ptr::null_mut();
    }

    if p_verbose_slang > 2 {
        verbose_enter_slang();
        smsg_slang(0, c"Reading spell file \"%s\"".as_ptr(), fname);
        verbose_leave_slang();
    }

    let lp: *mut crate::SlangRaw = if old_lp.is_null() {
        let new_lp = slang_alloc(lang);
        (*new_lp).sl_fname = xstrdup_slang(fname);
        // Check for .add.spl
        let tail = path_tail(fname);
        (*new_lp).sl_add = !libc::strstr(tail, SPL_FNAME_ADD.as_ptr().cast::<c_char>()).is_null();
        new_lp
    } else {
        old_lp
    };

    // Set sourcing_name so that error messages mention the file name.
    estack_push(ETYPE_SPELL_VAL, fname, 0);
    let did_estack_push = true;

    // Track whether we should free lp on error
    let allocated_lp = old_lp.is_null();

    // Helper: clean up and return null on failure
    macro_rules! fail {
        () => {{
            if !lang.is_null() {
                *lang = NUL as c_char;
            }
            if allocated_lp {
                slang_free(lp);
            }
            libc::fclose(fd);
            if did_estack_push {
                estack_pop();
            }
            return std::ptr::null_mut();
        }};
    }

    // <HEADER>: <fileID> -- validate magic string
    {
        let mut magic_buf = [0u8; VIMSPELLMAGICL];
        if libc::fread(magic_buf.as_mut_ptr().cast(), 1, VIMSPELLMAGICL, fd) != VIMSPELLMAGICL {
            if libc::feof(fd) != 0 {
                emsg_slang(c"E757: This does not look like a spell file".as_ptr());
            } else {
                let err = libc::strerror(libc::ferror(fd));
                semsg_slang(
                    c"E5042: Failed to read spell file %s: %s".as_ptr(),
                    fname,
                    err,
                );
            }
            fail!();
        }
        if magic_buf != *VIMSPELLMAGIC_BYTES {
            emsg_slang(c"E757: This does not look like a spell file".as_ptr());
            fail!();
        }
    }

    let c_ver = libc::fgetc(fd);
    if c_ver < VIMSPELLVERSION {
        emsg_slang(c"E771: Old spell file, needs to be updated".as_ptr());
        fail!();
    } else if c_ver > VIMSPELLVERSION {
        emsg_slang(c"E772: Spell file is for newer version of Vim".as_ptr());
        fail!();
    }

    // <SECTIONS>: <section> ... <sectionend>
    'sections: loop {
        let n = libc::fgetc(fd) as u8;
        if n == SN_END_V {
            break 'sections;
        }
        let sflags = libc::fgetc(fd) as u8; // <sectionflags>
        let len = get4c(fd); // <sectionlen>
        if len < 0 {
            emsg_slang(c"E758: Truncated spell file".as_ptr());
            fail!();
        }

        let mut res: c_int = 0;

        match n {
            SN_INFO_V => {
                if !(*lp).sl_info.is_null() {
                    xfree_slang((*lp).sl_info.cast::<c_void>());
                    (*lp).sl_info = std::ptr::null_mut();
                }
                (*lp).sl_info = read_string(fd, len as usize);
                if (*lp).sl_info.is_null() {
                    fail!();
                }
            }
            SN_REGION_V => {
                if len as usize > MAXREGIONS * 2 {
                    res = SP_FORMERROR;
                } else {
                    let rlen = len as usize;
                    if libc::fread((*lp).sl_regions.as_mut_ptr().cast(), 1, rlen, fd) != rlen {
                        res = if libc::feof(fd) != 0 {
                            SP_TRUNCERROR
                        } else {
                            SP_OTHERERROR
                        };
                    } else if !libc::memchr((*lp).sl_regions.as_ptr().cast(), 0, rlen).is_null() {
                        res = SP_FORMERROR;
                    } else {
                        (*lp).sl_regions[rlen] = 0;
                    }
                }
            }
            SN_CHARFLAGS_V => {
                // Read 1-byte flagslen
                let fc = libc::fgetc(fd);
                if fc == EOF {
                    res = SP_TRUNCERROR;
                } else {
                    let flagslen = fc as usize;
                    let flags_ptr: *mut c_char = if flagslen == 0 {
                        std::ptr::null_mut()
                    } else {
                        read_string(fd, flagslen)
                    };
                    if flagslen > 0 && flags_ptr.is_null() {
                        res = SP_OTHERERROR;
                    } else {
                        // Read 2-byte follen
                        let fc1 = libc::fgetc(fd);
                        let fc2 = libc::fgetc(fd);
                        if fc1 == EOF || fc2 == EOF {
                            if !flags_ptr.is_null() {
                                xfree_slang(flags_ptr.cast());
                            }
                            res = SP_TRUNCERROR;
                        } else {
                            let follen = ((fc1 as usize) << 8) | (fc2 as usize);
                            let fol_ptr: *mut c_char = if follen == 0 {
                                std::ptr::null_mut()
                            } else {
                                read_string(fd, follen)
                            };
                            if follen > 0 && fol_ptr.is_null() {
                                if !flags_ptr.is_null() {
                                    xfree_slang(flags_ptr.cast());
                                }
                                res = SP_OTHERERROR;
                            } else if flags_ptr.is_null() != fol_ptr.is_null() {
                                if !flags_ptr.is_null() {
                                    xfree_slang(flags_ptr.cast());
                                }
                                if !fol_ptr.is_null() {
                                    xfree_slang(fol_ptr.cast());
                                }
                                res = SP_FORMERROR;
                            } else if !flags_ptr.is_null() && !fol_ptr.is_null() {
                                let r = rs_set_spell_charflags(
                                    flags_ptr.cast::<u8>(),
                                    flagslen as c_int,
                                    fol_ptr,
                                );
                                if r != 0 {
                                    res = SP_OTHERERROR;
                                }
                                xfree_slang(flags_ptr.cast());
                                xfree_slang(fol_ptr.cast());
                            }
                        }
                    }
                }
            }
            SN_MIDWORD_V => {
                (*lp).sl_midword = read_string(fd, len as usize);
                if (*lp).sl_midword.is_null() {
                    fail!();
                }
            }
            SN_PREFCOND_V => {
                res = rs_read_prefcond_section(fd, lp);
            }
            SN_REP_V => {
                res = rs_read_rep_section(
                    fd,
                    std::ptr::addr_of_mut!((*lp).sl_rep),
                    (*lp).sl_rep_first.as_mut_ptr(),
                );
            }
            SN_REPSAL_V => {
                res = rs_read_rep_section(
                    fd,
                    std::ptr::addr_of_mut!((*lp).sl_repsal),
                    (*lp).sl_repsal_first.as_mut_ptr(),
                );
            }
            SN_SAL_V => {
                let sal_buf = xmalloc_slang(len as usize).cast::<u8>();
                if libc::fread(sal_buf.cast(), 1, len as usize, fd) != len as usize {
                    xfree_slang(sal_buf.cast());
                    res = if libc::feof(fd) != 0 {
                        SP_TRUNCERROR
                    } else {
                        SP_OTHERERROR
                    };
                } else {
                    res = rs_read_sal_section(sal_buf, len as usize, lp);
                    xfree_slang(sal_buf.cast());
                }
            }
            SN_SOFO_V => {
                (*lp).sl_sofo = true;
                if len <= 0 {
                    res = SP_FORMERROR;
                } else {
                    let sofo_buf = xmalloc_slang(len as usize).cast::<u8>();
                    if libc::fread(sofo_buf.cast(), 1, len as usize, fd) != len as usize {
                        xfree_slang(sofo_buf.cast());
                        res = if libc::feof(fd) != 0 {
                            SP_TRUNCERROR
                        } else {
                            SP_OTHERERROR
                        };
                    } else {
                        let mut sofo_section = SofoSection::default();
                        let mut sofo_consumed: usize = 0;
                        res = rs_parse_sofo_section(
                            sofo_buf,
                            len as usize,
                            std::ptr::addr_of_mut!(sofo_section),
                            std::ptr::addr_of_mut!(sofo_consumed),
                        );
                        xfree_slang(sofo_buf.cast());
                        if res == 0 {
                            let flen = sofo_section.from_len as usize;
                            let tlen = sofo_section.to_len as usize;
                            if flen == 0 && tlen == 0 {
                                // empty, OK
                            } else if flen == 0 || tlen == 0 {
                                res = SP_FORMERROR;
                            } else {
                                let mut sofo_from = [0u8; 513];
                                let mut sofo_to = [0u8; 513];
                                sofo_from[..flen].copy_from_slice(&sofo_section.from[..flen]);
                                sofo_to[..tlen].copy_from_slice(&sofo_section.to[..tlen]);
                                res = rs_set_sofo(
                                    lp,
                                    sofo_from.as_ptr().cast::<c_char>(),
                                    sofo_to.as_ptr().cast::<c_char>(),
                                );
                            }
                        }
                    }
                }
            }
            SN_MAP_V => {
                let p = read_string(fd, len as usize);
                if p.is_null() {
                    fail!();
                }
                rs_set_map_str(lp, p);
                xfree_slang(p.cast());
            }
            SN_WORDS_V => {
                let wbuf = xmalloc_slang(len as usize).cast::<u8>();
                if libc::fread(wbuf.cast(), 1, len as usize, fd) != len as usize {
                    xfree_slang(wbuf.cast());
                    res = if libc::feof(fd) != 0 {
                        SP_TRUNCERROR
                    } else {
                        SP_OTHERERROR
                    };
                    // fall through to error handling below
                } else {
                    let mut woff: usize = 0;
                    let mut wword = [0u8; MAXWLEN_V + 1];
                    let mut words_ok = true;
                    while woff < len as usize {
                        let mut wconsumed: usize = 0;
                        let wlen = rs_parse_words_entry(
                            wbuf.add(woff),
                            len as usize - woff,
                            wword.as_mut_ptr(),
                            MAXWLEN_V,
                            std::ptr::addr_of_mut!(wconsumed),
                        );
                        if wlen < 0 {
                            xfree_slang(wbuf.cast());
                            res = wlen;
                            words_ok = false;
                            break;
                        }
                        crate::rs_count_common_word(lp, wword.as_ptr().cast::<c_char>(), -1, 10);
                        woff += wconsumed;
                    }
                    if words_ok {
                        xfree_slang(wbuf.cast());
                    }
                }
            }
            SN_SUGFILE_V => {
                (*lp).sl_sugtime = get8ctime(fd);
            }
            SN_NOSPLITSUGS_V => {
                (*lp).sl_nosplitsugs = true;
            }
            SN_NOCOMPOUNDSUGS_V => {
                (*lp).sl_nocompoundsugs = true;
            }
            SN_COMPOUND_V => {
                let cmp_buf = xmalloc_slang(len as usize).cast::<u8>();
                if libc::fread(cmp_buf.cast(), 1, len as usize, fd) != len as usize {
                    xfree_slang(cmp_buf.cast());
                    res = if libc::feof(fd) != 0 {
                        SP_TRUNCERROR
                    } else {
                        SP_OTHERERROR
                    };
                } else {
                    res = rs_read_compound(cmp_buf, len as usize, lp);
                    xfree_slang(cmp_buf.cast());
                }
            }
            SN_NOBREAK_V => {
                (*lp).sl_nobreak = true;
            }
            SN_SYLLABLE_V => {
                (*lp).sl_syllable = read_string(fd, len as usize);
                if (*lp).sl_syllable.is_null() {
                    fail!();
                }
                if crate::rs_init_syl_tab(lp) != OK {
                    fail!();
                }
            }
            _ => {
                // Unsupported section: skip if not required, error if required.
                if sflags & SNF_REQUIRED_V != 0 {
                    emsg_slang(c"E770: Unsupported section in spell file".as_ptr());
                    fail!();
                }
                let mut skip = len;
                while skip > 0 {
                    skip -= 1;
                    if libc::fgetc(fd) < 0 {
                        emsg_slang(c"E758: Truncated spell file".as_ptr());
                        fail!();
                    }
                }
            }
        }

        // Handle section errors
        if res == SP_FORMERROR {
            emsg_slang(e_format_slang);
            fail!();
        }
        if res == SP_TRUNCERROR {
            emsg_slang(c"E758: Truncated spell file".as_ptr());
            fail!();
        }
        if res == SP_OTHERERROR {
            fail!();
        }
    }

    // Read all remaining tree data (<LWORDTREE> <KWORDTREE> <PREFIXTREE>).
    {
        let pos_before = libc::ftell(fd);
        libc::fseek(fd, 0, libc::SEEK_END);
        let pos_end = libc::ftell(fd);
        libc::fseek(fd, pos_before, libc::SEEK_SET);

        let tree_data_size = if pos_end > pos_before {
            (pos_end - pos_before) as usize
        } else {
            0
        };
        if tree_data_size == 0 {
            emsg_slang(c"E758: Truncated spell file".as_ptr());
            fail!();
        }

        let tree_data = xmalloc_slang(tree_data_size).cast::<u8>();
        if libc::fread(tree_data.cast(), 1, tree_data_size, fd) != tree_data_size {
            xfree_slang(tree_data.cast());
            emsg_slang(c"E758: Truncated spell file".as_ptr());
            fail!();
        }

        let mut toff: usize = 0;

        // Helper closure: read one tree from the buffer.
        // Returns false on error (after freeing tree_data and calling fail! logic).
        macro_rules! read_tree_from_buf {
            ($bytsp:expr, $bytsp_len_p:expr, $idxsp:expr, $is_prefix:expr, $prefcnt:expr) => {{
                let remaining = tree_data_size - toff;
                if remaining < 4 {
                    xfree_slang(tree_data.cast());
                    emsg_slang(c"E758: Truncated spell file".as_ptr());
                    fail!();
                }
                let mut nodecount: u32 = 0;
                if rs_read_tree_peek_nodecount(
                    tree_data.add(toff),
                    remaining,
                    std::ptr::addr_of_mut!(nodecount),
                ) != 0
                {
                    xfree_slang(tree_data.cast());
                    emsg_slang(c"E758: Truncated spell file".as_ptr());
                    fail!();
                }
                if nodecount == 0 {
                    $bytsp = std::ptr::null_mut();
                    if let Some(len_p) = $bytsp_len_p {
                        *len_p = 0;
                    }
                    $idxsp = std::ptr::null_mut();
                    toff += 4;
                } else {
                    let nc = nodecount as usize;
                    $bytsp = xmalloc_slang(nc).cast::<u8>();
                    if let Some(len_p) = $bytsp_len_p {
                        *len_p = nc as c_int;
                    }
                    $idxsp = xcalloc_slang(nc, std::mem::size_of::<c_int>()).cast::<c_int>();
                    let mut consumed: usize = 0;
                    let mut nc_out: c_int = 0;
                    let r = rs_read_tree(
                        tree_data.add(toff),
                        remaining,
                        $bytsp,
                        $idxsp as *mut i32,
                        nc,
                        $is_prefix,
                        $prefcnt,
                        std::ptr::addr_of_mut!(consumed),
                        std::ptr::addr_of_mut!(nc_out),
                    );
                    if r != 0 {
                        xfree_slang(tree_data.cast());
                        // error handling
                        if r == SP_FORMERROR {
                            emsg_slang(e_format_slang);
                        } else {
                            emsg_slang(c"E758: Truncated spell file".as_ptr());
                        }
                        fail!();
                    }
                    toff += consumed;
                }
            }};
        }

        // <LWORDTREE>
        read_tree_from_buf!(
            (*lp).sl_fbyts,
            Some(std::ptr::addr_of_mut!((*lp).sl_fbyts_len)),
            (*lp).sl_fidxs,
            false,
            0
        );
        // <KWORDTREE>
        read_tree_from_buf!((*lp).sl_kbyts, None::<*mut c_int>, (*lp).sl_kidxs, false, 0);
        // <PREFIXTREE>
        let prefcnt = (*lp).sl_prefixcnt;
        read_tree_from_buf!(
            (*lp).sl_pbyts,
            None::<*mut c_int>,
            (*lp).sl_pidxs,
            true,
            prefcnt
        );

        xfree_slang(tree_data.cast());
    }

    // For a new file, link it into the list of spell files.
    if old_lp.is_null() && !lang.is_null() {
        (*lp).sl_next = first_lang_mut;
        first_lang_mut = lp;
    }

    // Success path
    libc::fclose(fd);
    if did_estack_push {
        estack_pop();
    }
    lp
}

/// Reload the spell file `fname` if it's already loaded.
///
/// Called after writing a new .add.spl (from mkspell / spell_add_word).
/// `added_word` is true when invoked via "zg".
///
/// # Safety
/// `fname` must be a valid NUL-terminated path string.
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_spell_reload_one(fname: *mut c_char, added_word: bool) {
    let mut didit = false;

    let mut slang = first_lang_mut;
    while !slang.is_null() {
        if path_full_compare_slang(fname, (*slang).sl_fname, false, true) == K_EQUAL_FILES_SLANG {
            slang_clear(slang);
            if rs_spell_load_file(fname, std::ptr::null_mut(), slang, false).is_null() {
                // reloading failed, clear the language
                slang_clear(slang);
            }
            redraw_all_later(UPD_SOME_VALID);
            didit = true;
        }
        slang = (*slang).sl_next;
    }

    // When "zg" was used and the file wasn't loaded yet, redo 'spelllang' to load it.
    if added_word && !didit {
        parse_spelllang_slang(curwin_slang);
    }
}

/// Load the .sug files for languages that have one and weren't loaded yet.
///
/// Iterates over the current window's b_langp array, and for each language that
/// has a .sug timestamp but has not been loaded yet, loads the .sug file.
///
/// # Safety
/// Must be called with a valid curwin.
#[export_name = "suggest_load_files"]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::too_many_lines,
    clippy::if_not_else,
    clippy::needless_continue,
    clippy::useless_let_if_seq
)]
pub unsafe extern "C" fn rs_suggest_load_files() {
    const VIMSUGMAGIC_BYTES: &[u8] = b"VIMsug";
    const VIMSUGMAGICL: usize = 6;
    const VIMSUGVERSION_V: c_int = 1;
    const SP_TRUNCERROR_V: c_int = -1;
    const SP_FORMERROR_V: c_int = -2;
    const SP_OTHERERROR_V: c_int = -3;
    const FAIL_V: c_int = 1;

    let langp_ga = nvim_win_get_b_langp(curwin_slang);
    let lpi_count = (*langp_ga).ga_len;

    for lpi in 0..lpi_count {
        let lp = crate::langp_entry(langp_ga, lpi);
        let slang = (*lp).lp_slang;
        if (*slang).sl_sugtime == 0 || (*slang).sl_sugloaded {
            continue;
        }

        // Mark as loaded so we don't retry on failure.
        (*slang).sl_sugloaded = true;

        // Find the '.' that precedes the extension.
        let dotp = libc::strrchr((*slang).sl_fname, b'.' as c_int);
        if dotp.is_null() || path_fnamecmp(dotp, c".spl".as_ptr()) != 0 {
            continue;
        }

        // Swap ".spl" -> ".sug" in-place.
        libc::strcpy(dotp, c".sug".as_ptr());

        let fd = os_fopen_slang((*slang).sl_fname, c"r".as_ptr());

        // Closure to restore ".spl" and optionally close fd.
        macro_rules! next_one {
            ($fd_opt:expr) => {{
                if let Some(fdc) = $fd_opt {
                    libc::fclose(fdc);
                }
                libc::strcpy(dotp, c".spl".as_ptr());
                continue;
            }};
        }

        if fd.is_null() {
            next_one!(None::<*mut libc::FILE>);
        }

        // <SUGHEADER>: <fileID> <versionnr> <timestamp>
        let mut hdr_buf = [0u8; VIMSUGMAGICL];
        for slot in &mut hdr_buf {
            let c = libc::fgetc(fd);
            if c < 0 {
                next_one!(Some(fd));
            }
            *slot = c as u8;
        }
        if &hdr_buf[..VIMSUGMAGICL] != VIMSUGMAGIC_BYTES {
            semsg_slang(
                c"E778: This does not look like a .sug file: %s".as_ptr(),
                (*slang).sl_fname,
            );
            next_one!(Some(fd));
        }

        let c_ver = libc::fgetc(fd);
        if c_ver < VIMSUGVERSION_V {
            semsg_slang(
                c"E779: Old .sug file, needs to be updated: %s".as_ptr(),
                (*slang).sl_fname,
            );
            next_one!(Some(fd));
        } else if c_ver > VIMSUGVERSION_V {
            semsg_slang(
                c"E780: .sug file is for newer version of Vim: %s".as_ptr(),
                (*slang).sl_fname,
            );
            next_one!(Some(fd));
        }

        // Check timestamp: must match .spl timestamp exactly.
        let timestamp = get8ctime(fd);
        if timestamp != (*slang).sl_sugtime {
            semsg_slang(
                c"E781: .sug file doesn't match .spl file: %s".as_ptr(),
                (*slang).sl_fname,
            );
            next_one!(Some(fd));
        }

        // Helper macro: report error, clear sug data, and go to next.
        macro_rules! somerror {
            ($fd_val:expr) => {{
                semsg_slang(
                    c"E782: Error while reading .sug file: %s".as_ptr(),
                    (*slang).sl_fname,
                );
                slang_clear_sug(slang);
                next_one!(Some($fd_val));
            }};
        }

        // <SUGWORDTREE>: read the soundfold trie.
        {
            let mut rt_hdr = [0u8; 4];
            let mut rt_res: c_int = 0;
            if libc::fread(rt_hdr.as_mut_ptr().cast(), 1, 4, fd) != 4 {
                rt_res = if libc::feof(fd) != 0 {
                    SP_TRUNCERROR_V
                } else {
                    SP_OTHERERROR_V
                };
            } else {
                let rt_len = ((rt_hdr[0] as i32) << 24)
                    | ((rt_hdr[1] as i32) << 16)
                    | ((rt_hdr[2] as i32) << 8)
                    | rt_hdr[3] as i32;
                if rt_len < 0 {
                    rt_res = SP_TRUNCERROR_V;
                } else if rt_len as usize >= usize::MAX / std::mem::size_of::<c_int>() {
                    rt_res = SP_FORMERROR_V;
                } else if rt_len > 0 {
                    let rt_bp = xmalloc_slang(rt_len as usize).cast::<u8>();
                    (*slang).sl_sbyts = rt_bp;
                    let rt_sidxs = xcalloc_slang(rt_len as usize, std::mem::size_of::<c_int>())
                        .cast::<c_int>();
                    (*slang).sl_sidxs = rt_sidxs;
                    // Allocate a reading buffer: header + up to rt_len*6 + 64 bytes.
                    let rt_buf_max = 4 + (rt_len as usize) * 6 + 64;
                    let rt_buf = xmalloc_slang(rt_buf_max).cast::<u8>();
                    libc::memcpy(rt_buf.cast(), rt_hdr.as_ptr().cast(), 4);
                    let rt_data = libc::fread(rt_buf.add(4).cast(), 1, rt_buf_max - 4, fd);
                    if rt_data == 0 {
                        xfree_slang(rt_buf.cast());
                        rt_res = SP_TRUNCERROR_V;
                    } else {
                        let mut rt_consumed: usize = 0;
                        let mut rt_nc: c_int = 0;
                        rt_res = rs_read_tree(
                            rt_buf,
                            4 + rt_data,
                            rt_bp,
                            rt_sidxs,
                            rt_len as usize,
                            false,
                            0,
                            std::ptr::addr_of_mut!(rt_consumed),
                            std::ptr::addr_of_mut!(rt_nc),
                        );
                        let rt_over = (4 + rt_data) as i64 - rt_consumed as i64;
                        if rt_over > 0 {
                            libc::fseek(fd, -rt_over, libc::SEEK_CUR);
                        }
                        xfree_slang(rt_buf.cast());
                    }
                }
            }
            if rt_res != 0 {
                somerror!(fd);
            }
        }

        // <SUGTABLE>: <sugwcount> <sugline> ...
        (*slang).sl_sugbuf = open_spellbuf();

        let wcount = get4c(fd);
        if wcount < 0 {
            somerror!(fd);
        }

        // Read all the wordnr lists into the buffer, one NUL-terminated list per line.
        let mut ga = crate::GArrayRaw {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 1,
            ga_growsize: 100,
            ga_data: std::ptr::null_mut(),
        };

        for wordnr in 0..wcount {
            ga.ga_len = 0;
            loop {
                let c = libc::fgetc(fd);
                if c < 0 {
                    ga_clear_sug(std::ptr::addr_of_mut!(ga));
                    somerror!(fd);
                }
                // GA_APPEND equivalent: grow by 1 and append the byte.
                ga_grow(std::ptr::addr_of_mut!(ga), 1);
                let slot = (ga.ga_data as *mut u8).add(ga.ga_len as usize);
                *slot = c as u8;
                ga.ga_len += 1;
                if c == 0 {
                    break;
                }
            }
            if ml_append_buf(
                (*slang).sl_sugbuf,
                wordnr,
                ga.ga_data.cast::<c_char>(),
                ga.ga_len,
                true,
            ) == FAIL_V
            {
                ga_clear_sug(std::ptr::addr_of_mut!(ga));
                somerror!(fd);
            }
        }
        ga_clear_sug(std::ptr::addr_of_mut!(ga));

        // Count words in the tries so they can be found by word number.
        crate::rs_tree_count_words((*slang).sl_fbyts, (*slang).sl_fidxs, (*slang).sl_fbyts_len);
        // Soundfold tree has no stored length; use i32::MAX.
        crate::rs_tree_count_words((*slang).sl_sbyts, (*slang).sl_sidxs, c_int::MAX);

        next_one!(Some(fd));
    }
}

/// Orchestrate .sug file creation for a spell language.
///
/// # Safety
/// `spin` and `wfname` must be valid non-null pointers.
#[export_name = "spell_make_sugfile"]
#[inline(never)]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation
)]
pub unsafe extern "C" fn rs_spell_make_sugfile(spin: *mut SpellinfoT, wfname: *mut c_char) {
    const OK: c_int = 0;

    let mut slang = first_lang_sug;
    while !slang.is_null() {
        if path_full_compare_sug(wfname, (*slang).sl_fname, false, true) == K_EQUAL_FILES {
            break;
        }
        slang = (*slang).sl_next;
    }
    let free_slang = slang.is_null();
    if free_slang {
        rs_spell_message(spin, c"Reading back spell file...".as_ptr());
        slang = rs_spell_load_file(wfname, std::ptr::null_mut(), std::ptr::null_mut(), false);
        if slang.is_null() {
            return;
        }
    }

    (*spin).si_blocks = std::ptr::null_mut();
    (*spin).si_blocks_cnt = 0;
    (*spin).si_compress_cnt = 0;
    (*spin).si_free_count = 0;
    (*spin).si_first_free = std::ptr::null_mut();
    (*spin).si_foldwcount = 0;

    rs_spell_message(spin, c"Performing soundfolding...".as_ptr());
    if sug_filltree(spin, slang) != OK {
        if free_slang {
            slang_free(slang);
        }
        free_blocks((*spin).si_blocks.cast::<c_void>());
        close_spellbuf((*spin).si_spellbuf);
        return;
    }

    if sug_maketable(spin) != OK {
        if free_slang {
            slang_free(slang);
        }
        free_blocks((*spin).si_blocks.cast::<c_void>());
        close_spellbuf((*spin).si_spellbuf);
        return;
    }

    smsg_sug(
        0,
        c"Number of words after soundfolding: %ld".as_ptr(),
        nvim_decor_buf_get_line_count((*spin).si_spellbuf) as libc::c_long,
    );

    rs_spell_message(spin, c"Compressing word tree...".as_ptr());
    rs_wordtree_compress_export(spin, (*spin).si_foldroot, c"case-folded".as_ptr());

    let wfname_len = libc::strlen(wfname);
    let fname_buf = libc::malloc(wfname_len + 1).cast::<u8>();
    if fname_buf.is_null() {
        if free_slang {
            slang_free(slang);
        }
        free_blocks((*spin).si_blocks.cast::<c_void>());
        close_spellbuf((*spin).si_spellbuf);
        return;
    }
    libc::memcpy(
        fname_buf.cast::<c_void>(),
        wfname.cast::<c_void>(),
        wfname_len + 1,
    );
    *fname_buf.add(wfname_len - 2) = b'u';
    *fname_buf.add(wfname_len - 1) = b'g';
    sug_write(spin, fname_buf.cast::<c_char>());
    libc::free(fname_buf.cast::<c_void>());

    if free_slang {
        slang_free(slang);
    }
    free_blocks((*spin).si_blocks.cast::<c_void>());
    close_spellbuf((*spin).si_spellbuf);
}

// =============================================================================
// Phase 1: Small pure helpers and affix flag utilities
// =============================================================================

/// AFT_ flag type constants (match C defines in spellfile.c).
const AFT_CHAR: c_int = 0; // flags are one character
const AFT_LONG: c_int = 1; // flags are two characters
const AFT_CAPLONG: c_int = 2; // flags are one or two characters
const AFT_NUM: c_int = 3; // flags are numbers, comma separated

/// ZERO_FLAG: used when a numeric flag is zero (value "0").
const ZERO_FLAG: u32 = 65009;

/// AH_KEY_LEN: 2 x 8 bytes + NUL.
const AH_KEY_LEN: usize = 17;

extern "C" {
    /// smsg(hl_id, fmt, ...): display a message.
    #[link_name = "smsg"]
    fn smsg_phase1(hl_id: c_int, fmt: *const c_char, ...) -> c_int;
    /// getdigits_int: parse a decimal integer from *pp, advancing the pointer.
    fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int;
    /// ascii_isdigit: returns true if char is ASCII digit.
    #[link_name = "rs_ascii_isdigit"]
    fn ascii_isdigit(c: c_int) -> c_int;
    /// hash_find: look up key in hashtable, returns pointer to hashitem (or empty item).
    fn hash_find(ht: *const crate::HashtabRaw, key: *const c_char) -> *mut crate::HashitemRaw;
}

/// Check if a hashitem is empty (key is NULL or hash_removed sentinel).
///
/// # Safety
/// `hi` must be a valid non-null pointer to a `HashitemRaw`.
unsafe fn hi_is_empty(hi: *const crate::HashitemRaw) -> bool {
    (*hi).hi_key.is_null()
        || std::ptr::eq(
            (*hi).hi_key,
            std::ptr::addr_of!(hash_removed_sentinel).cast::<c_char>(),
        )
}

/// afffile_T repr(C) mirror (64-bit layout from spellfile.c).
///
/// sizeof = 952, alignof = 8
///
/// Offsets:
///   0: af_enc (ptr), 8: af_flagtype (int), 12: af_rare..af_nosuggest (10 x u32)
///   52: af_pfxpostpone (int), 56: af_ignoreextra (bool), 57..63: pad
///   64: af_pref (hashtab, 296 bytes), 360: af_suff (296), 656: af_comp (296)
#[repr(C)]
struct AfffileT {
    pub af_enc: *mut c_char,        // offset 0
    pub af_flagtype: c_int,         // offset 8
    pub af_rare: u32,               // offset 12
    pub af_keepcase: u32,           // offset 16
    pub af_bad: u32,                // offset 20
    pub af_needaffix: u32,          // offset 24
    pub af_circumfix: u32,          // offset 28
    pub af_needcomp: u32,           // offset 32
    pub af_comproot: u32,           // offset 36
    pub af_compforbid: u32,         // offset 40
    pub af_comppermit: u32,         // offset 44
    pub af_nosuggest: u32,          // offset 48
    pub af_pfxpostpone: c_int,      // offset 52
    pub af_ignoreextra: bool,       // offset 56
    _pad0: [u8; 7],                 // offset 57
    pub af_pref: crate::HashtabRaw, // offset 64 (296 bytes)
    pub af_suff: crate::HashtabRaw, // offset 360 (296 bytes)
    pub af_comp: crate::HashtabRaw, // offset 656 (296 bytes)
}

const _: () = {
    assert!(std::mem::size_of::<AfffileT>() == 952);
    assert!(std::mem::align_of::<AfffileT>() == 8);
    assert!(std::mem::offset_of!(AfffileT, af_enc) == 0);
    assert!(std::mem::offset_of!(AfffileT, af_flagtype) == 8);
    assert!(std::mem::offset_of!(AfffileT, af_rare) == 12);
    assert!(std::mem::offset_of!(AfffileT, af_nosuggest) == 48);
    assert!(std::mem::offset_of!(AfffileT, af_pfxpostpone) == 52);
    assert!(std::mem::offset_of!(AfffileT, af_ignoreextra) == 56);
    assert!(std::mem::offset_of!(AfffileT, af_pref) == 64);
    assert!(std::mem::offset_of!(AfffileT, af_suff) == 360);
    assert!(std::mem::offset_of!(AfffileT, af_comp) == 656);
};

/// affentry_T repr(C) mirror.
///
/// sizeof = 56, alignof = 8
#[repr(C)]
struct AffentryT {
    pub ae_next: *mut AffentryT, // offset 0
    pub ae_chop: *mut c_char,    // offset 8
    pub ae_add: *mut c_char,     // offset 16
    pub ae_flags: *mut c_char,   // offset 24
    pub ae_cond: *mut c_char,    // offset 32
    pub ae_prog: *mut c_void,    // offset 40 (regprog_T*)
    pub ae_compforbid: c_char,   // offset 48
    pub ae_comppermit: c_char,   // offset 49
    _pad0: [u8; 6],              // offset 50
}

const _: () = {
    assert!(std::mem::size_of::<AffentryT>() == 56);
    assert!(std::mem::offset_of!(AffentryT, ae_next) == 0);
    assert!(std::mem::offset_of!(AffentryT, ae_add) == 16);
    assert!(std::mem::offset_of!(AffentryT, ae_flags) == 24);
    assert!(std::mem::offset_of!(AffentryT, ae_prog) == 40);
    assert!(std::mem::offset_of!(AffentryT, ae_compforbid) == 48);
    assert!(std::mem::offset_of!(AffentryT, ae_comppermit) == 49);
};

/// affheader_T repr(C) mirror (accessed via HI2AH: `(affheader_T *)(hi)->hi_key`).
///
/// sizeof = 48, alignof = 8
#[repr(C)]
#[allow(non_snake_case)]
struct AffheaderT {
    pub ah_key: [c_char; AH_KEY_LEN], // offset 0
    _pad0: [u8; 3],                   // offset 17
    pub ah_flag: u32,                 // offset 20
    pub ah_newID: c_int,              // offset 24
    pub ah_combine: c_int,            // offset 28
    pub ah_follows: c_int,            // offset 32
    _pad1: [u8; 4],                   // offset 36
    pub ah_first: *mut AffentryT,     // offset 40
}

const _: () = {
    assert!(std::mem::size_of::<AffheaderT>() == 48);
    assert!(std::mem::offset_of!(AffheaderT, ah_key) == 0);
    assert!(std::mem::offset_of!(AffheaderT, ah_flag) == 20);
    assert!(std::mem::offset_of!(AffheaderT, ah_newID) == 24);
    assert!(std::mem::offset_of!(AffheaderT, ah_first) == 40);
};

/// compitem_T repr(C) mirror (accessed via HI2CI: `(compitem_T *)(hi)->hi_key`).
///
/// sizeof = 28, alignof = 4
#[repr(C)]
#[allow(non_snake_case)]
struct CompitemT {
    pub ci_key: [c_char; AH_KEY_LEN], // offset 0
    _pad0: [u8; 3],                   // offset 17
    pub ci_flag: u32,                 // offset 20
    pub ci_newID: c_int,              // offset 24
}

const _: () = {
    assert!(std::mem::offset_of!(CompitemT, ci_key) == 0);
    assert!(std::mem::offset_of!(CompitemT, ci_flag) == 20);
    assert!(std::mem::offset_of!(CompitemT, ci_newID) == 24);
};

/// Get one affix name from `*pp` and advance the pointer.
/// Returns `ZERO_FLAG` for "0", 0 for error (still advances pointer).
/// Mirrors C `get_affitem`.
///
/// # Safety
/// `pp` must be a valid non-null double pointer to a NUL-terminated C string.
#[allow(clippy::cast_sign_loss)]
unsafe fn get_affitem_inner(flagtype: c_int, pp: *mut *mut c_char) -> u32 {
    if flagtype == AFT_NUM {
        if ascii_isdigit(i32::from(**pp as u8)) == 0 {
            *pp = (*pp).add(1); // always advance, avoid getting stuck
            return 0;
        }
        let res = getdigits_int(pp, true, 0);
        if res == 0 {
            ZERO_FLAG
        } else {
            res as u32
        }
    } else {
        let res = mb_ptr2char_adv_p(pp.cast::<*const c_char>()) as u32;
        if flagtype == AFT_LONG
            || (flagtype == AFT_CAPLONG && res >= u32::from(b'A') && res <= u32::from(b'Z'))
        {
            if **pp == 0 {
                return 0;
            }
            let res2 = mb_ptr2char_adv_p(pp.cast::<*const c_char>()) as u32;
            res2 + (res << 16)
        } else {
            res
        }
    }
}

/// FFI export of `get_affitem` for C callers.
///
/// # Safety
/// `pp` must be a valid non-null double pointer to a NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_affitem(flagtype: c_int, pp: *mut *mut c_char) -> u32 {
    get_affitem_inner(flagtype, pp)
}

/// Turn an affix flag name into a number, according to the FLAG type.
/// Returns zero for failure.
/// Mirrors C `affitem2flag`.
///
/// # Safety
/// `item` and `fname` must be valid NUL-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_affitem2flag(
    flagtype: c_int,
    item: *mut c_char,
    fname: *const c_char,
    lnum: c_int,
) -> u32 {
    let mut p = item;
    let res = get_affitem_inner(flagtype, &raw mut p);
    if res == 0 {
        if flagtype == AFT_NUM {
            smsg_phase1(
                0,
                c"Flag is not a number in %s line %d: %s".as_ptr(),
                fname,
                lnum,
                item,
            );
        } else {
            smsg_phase1(
                0,
                c"Illegal flag in %s line %d: %s".as_ptr(),
                fname,
                lnum,
                item,
            );
        }
    }
    if *p != 0 {
        smsg_phase1(
            0,
            c"Affix name too long in %s line %d: %s".as_ptr(),
            fname,
            lnum,
            item,
        );
        return 0;
    }
    res
}

/// Check if flag `flag` appears in affix list `afflist`.
/// Mirrors C `flag_in_afflist`.
///
/// # Safety
/// `afflist` must be a valid NUL-terminated C string.
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
unsafe fn flag_in_afflist_inner(flagtype: c_int, afflist: *mut c_char, flag: u32) -> bool {
    match flagtype {
        AFT_CHAR => !crate::vim_strchr(afflist, flag as c_int).is_null(),
        AFT_CAPLONG | AFT_LONG => {
            let mut p = afflist;
            while *p != 0 {
                let mut n = mb_ptr2char_adv_p((&raw mut p).cast::<*const c_char>()) as u32;
                if (flagtype == AFT_LONG || (n >= u32::from(b'A') && n <= u32::from(b'Z')))
                    && *p != 0
                {
                    n = mb_ptr2char_adv_p((&raw mut p).cast::<*const c_char>()) as u32 + (n << 16);
                }
                if n == flag {
                    return true;
                }
            }
            false
        }
        AFT_NUM => {
            let mut p = afflist;
            while *p != 0 {
                let digits = getdigits_int((&raw mut p).cast::<*mut c_char>(), true, 0);
                assert!(digits >= 0);
                let mut n = digits as u32;
                if n == 0 {
                    n = ZERO_FLAG;
                }
                if n == flag {
                    return true;
                }
                if *p != 0 {
                    p = p.add(1); // skip over comma
                }
            }
            false
        }
        _ => false,
    }
}

/// FFI export of `flag_in_afflist` for C callers.
///
/// # Safety
/// `afflist` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_flag_in_afflist(
    flagtype: c_int,
    afflist: *mut c_char,
    flag: u32,
) -> bool {
    flag_in_afflist_inner(flagtype, afflist, flag)
}

/// Give a warning when `spinval` and `affval` numbers are set and not the same.
/// Mirrors C `aff_check_number`.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_aff_check_number(spinval: c_int, affval: c_int, name: *const c_char) {
    if spinval != 0 && spinval != affval {
        smsg_phase1(
            0,
            c"%s value differs from what is used in another .aff file".as_ptr(),
            name,
        );
    }
}

/// Give a warning when `spinval` and `affval` strings are set and not the same.
/// Mirrors C `aff_check_string`.
///
/// # Safety
/// All pointers must be valid NUL-terminated C strings (or `spinval` may be NULL).
#[no_mangle]
pub unsafe extern "C" fn rs_aff_check_string(
    spinval: *const c_char,
    affval: *const c_char,
    name: *const c_char,
) {
    if spinval.is_null() {
        return;
    }
    // strcmp: check if strings differ
    let mut s1 = spinval;
    let mut s2 = affval;
    loop {
        if *s1 != *s2 {
            smsg_phase1(
                0,
                c"%s value differs from what is used in another .aff file".as_ptr(),
                name,
            );
            return;
        }
        if *s1 == 0 {
            return;
        }
        s1 = s1.add(1);
        s2 = s2.add(1);
    }
}

/// Check that new IDs for postponed affixes and compounding don't overrun each other.
/// Mirrors C `check_renumber`.
///
/// # Safety
/// `spin` must be a valid non-null pointer to a `spellinfo_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_check_renumber(spin: *mut SpellinfoT) {
    if (*spin).si_newprefID == (*spin).si_newcompID && (*spin).si_newcompID < 128 {
        (*spin).si_newprefID = 127;
        (*spin).si_newcompID = 255;
    }
}

/// Extract WF_ flags from an affix list.
/// Mirrors C `get_affix_flags`.
///
/// # Safety
/// `affile` and `afflist` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_get_affix_flags(affile: *mut AfffileT, afflist: *mut c_char) -> c_int {
    // WF_ flag constants (from spell_defs.h)
    const WF_KEEPCAP: c_int = 0x0400;
    const WF_FIXCAP: c_int = 0x4000;
    const WF_RARE: c_int = 0x0200;
    const WF_BANNED: c_int = 0x0010;
    const WF_NEEDCOMP: c_int = 0x1000;
    const WF_COMPROOT: c_int = 0x0800;
    const WF_NOSUGGEST: c_int = 0x2000;

    let mut flags: c_int = 0;
    let ft = (*affile).af_flagtype;

    if (*affile).af_keepcase != 0 && flag_in_afflist_inner(ft, afflist, (*affile).af_keepcase) {
        flags |= WF_KEEPCAP | WF_FIXCAP;
    }
    if (*affile).af_rare != 0 && flag_in_afflist_inner(ft, afflist, (*affile).af_rare) {
        flags |= WF_RARE;
    }
    if (*affile).af_bad != 0 && flag_in_afflist_inner(ft, afflist, (*affile).af_bad) {
        flags |= WF_BANNED;
    }
    if (*affile).af_needcomp != 0 && flag_in_afflist_inner(ft, afflist, (*affile).af_needcomp) {
        flags |= WF_NEEDCOMP;
    }
    if (*affile).af_comproot != 0 && flag_in_afflist_inner(ft, afflist, (*affile).af_comproot) {
        flags |= WF_COMPROOT;
    }
    if (*affile).af_nosuggest != 0 && flag_in_afflist_inner(ft, afflist, (*affile).af_nosuggest) {
        flags |= WF_NOSUGGEST;
    }
    flags
}

/// Get list of prefix IDs from affix list (for PFXPOSTPONE).
/// Writes IDs into `store_afflist` (NUL-terminated), returns count.
/// Mirrors C `get_pfxlist`.
///
/// # Safety
/// All pointers must be valid. `store_afflist` must have at least MAXWLEN bytes.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn rs_get_pfxlist(
    affile: *mut AfffileT,
    afflist: *mut c_char,
    store_afflist: *mut c_char,
) -> c_int {
    let mut cnt: c_int = 0;
    let mut key = [0u8; AH_KEY_LEN];
    let ft = (*affile).af_flagtype;

    let mut p = afflist;
    while *p != 0 {
        let prevp = p;
        if get_affitem_inner(ft, &raw mut p) != 0 {
            // Copy flag text (prevp..p) into key buffer.
            #[allow(clippy::cast_sign_loss)]
            let key_len = p.offset_from(prevp) as usize;
            let copy_len = key_len.min(AH_KEY_LEN - 1);
            std::ptr::copy_nonoverlapping(prevp.cast::<u8>(), key.as_mut_ptr(), copy_len);
            key[copy_len] = 0;

            let hi = hash_find(&raw const (*affile).af_pref, key.as_ptr().cast::<c_char>());
            if !hi.is_null() && !hi_is_empty(hi) {
                // HI2AH: hi_key points to start of affheader_T (key is first field).
                // The C struct has the key as its first field (offset 0), so the pointer
                // is valid as affheader_T* regardless of alignment lint.
                #[allow(clippy::cast_ptr_alignment)]
                let ah = (*hi).hi_key.cast::<AffheaderT>();
                let id = (*ah).ah_newID;
                if id != 0 {
                    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                    let byte = id as u8;
                    #[allow(clippy::cast_sign_loss)]
                    let idx = cnt as usize;
                    *store_afflist.add(idx) = byte as c_char;
                    cnt += 1;
                }
            }
        }
        #[allow(clippy::cast_possible_wrap)]
        if ft == AFT_NUM && *p == b',' as c_char {
            p = p.add(1);
        }
    }
    #[allow(clippy::cast_sign_loss)]
    let end_idx = cnt as usize;
    *store_afflist.add(end_idx) = 0;
    cnt
}

/// Get list of compound IDs from affix list.
/// Writes IDs into `store_afflist` (NUL-terminated).
/// Mirrors C `get_compflags`.
///
/// # Safety
/// All pointers must be valid. `store_afflist` must have at least MAXWLEN bytes.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_ptr_alignment
)]
pub unsafe extern "C" fn rs_get_compflags(
    affile: *mut AfffileT,
    afflist: *mut c_char,
    store_afflist: *mut c_char,
) {
    let mut cnt: usize = 0;
    let mut key = [0u8; AH_KEY_LEN];
    let ft = (*affile).af_flagtype;

    let mut p = afflist;
    while *p != 0 {
        let prevp = p;
        if get_affitem_inner(ft, &raw mut p) != 0 {
            let key_len = p.offset_from(prevp) as usize;
            let copy_len = key_len.min(AH_KEY_LEN - 1);
            std::ptr::copy_nonoverlapping(prevp.cast::<u8>(), key.as_mut_ptr(), copy_len);
            key[copy_len] = 0;

            let hi = hash_find(&raw const (*affile).af_comp, key.as_ptr().cast::<c_char>());
            if !hi.is_null() && !hi_is_empty(hi) {
                // HI2CI: hi_key points to start of compitem_T (key is first field).
                let ci = (*hi).hi_key.cast::<CompitemT>();
                *store_afflist.add(cnt) = (*ci).ci_newID as u8 as c_char;
                cnt += 1;
            }
        }
        if ft == AFT_NUM && *p == b',' as c_char {
            p = p.add(1);
        }
    }
    *store_afflist.add(cnt) = 0;
}

/// Process COMPOUNDFORBIDFLAG/COMPOUNDPERMITFLAG in an affix entry.
/// Removes matched flags from ae_flags and sets ae_compforbid/ae_comppermit.
/// Mirrors C `aff_process_flags`.
///
/// # Safety
/// `affile` and `entry` must be valid non-null pointers.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_aff_process_flags(affile: *mut AfffileT, entry: *mut AffentryT) {
    if (*entry).ae_flags.is_null() {
        return;
    }
    if (*affile).af_compforbid == 0 && (*affile).af_comppermit == 0 {
        return;
    }

    let ft = (*affile).af_flagtype;
    let mut p = (*entry).ae_flags;
    while *p != 0 {
        let prevp = p;
        let flag = get_affitem_inner(ft, &raw mut p);
        if flag == (*affile).af_comppermit || flag == (*affile).af_compforbid {
            // STRMOVE(prevp, p): memmove to shift string left and remove this flag.
            let move_len = libc::strlen(p.cast::<libc::c_char>()) + 1;
            libc::memmove(
                prevp.cast::<libc::c_void>(),
                p.cast::<libc::c_void>(),
                move_len,
            );
            p = prevp;
            if flag == (*affile).af_comppermit {
                (*entry).ae_comppermit = 1;
            } else {
                (*entry).ae_compforbid = 1;
            }
        }
        if ft == AFT_NUM && *p == b',' as c_char {
            p = p.add(1);
        }
    }
    if *(*entry).ae_flags == 0 {
        (*entry).ae_flags = std::ptr::null_mut(); // nothing left
    }
}

// =============================================================================
// Phase 2: Arena allocator and memory management
// =============================================================================

extern "C" {
    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn spell_casefold(
        wp: *const c_void,
        str_: *const c_char,
        len: c_int,
        buf: *mut c_char,
        buflen: c_int,
    ) -> c_int;
}

/// Arena allocator: returns a zeroed sub-allocation of `len` bytes from
/// `spin->si_blocks`. Mirrors C `getroom`.
///
/// # Panics
/// Panics if `len > SBLOCKSIZE`.
///
/// # Safety
/// `spin` must be a valid non-null pointer to a `spellinfo_T`.
#[export_name = "getroom"]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn rs_getroom(spin: *mut SpellinfoT, len: usize, align: bool) -> *mut c_void {
    assert!(len <= SBLOCKSIZE, "getroom: len > SBLOCKSIZE");

    let mut bl = (*spin).si_blocks;

    if align && !bl.is_null() {
        // Round sb_used up to pointer alignment.
        let align_mask = std::mem::align_of::<*mut c_void>() - 1;
        (*bl).sb_used = (((*bl).sb_used as usize + align_mask) & !align_mask) as c_int;
    }

    if bl.is_null() || ((*bl).sb_used as usize) + len > SBLOCKSIZE {
        // Allocate a new block (header + SBLOCKSIZE + 1 bytes for data).
        let alloc_size = std::mem::size_of::<SblockT>() + SBLOCKSIZE + 1;
        let new_bl = xcalloc(1, alloc_size).cast::<SblockT>();
        (*new_bl).sb_next = (*spin).si_blocks;
        (*spin).si_blocks = new_bl;
        (*new_bl).sb_used = 0;
        (*spin).si_blocks_cnt += 1;
        bl = new_bl;
    }

    // sb_data[] starts immediately after the SblockT header.
    let data_start = bl.add(1).cast::<u8>();
    let p = data_start.add((*bl).sb_used as usize);
    (*bl).sb_used += len as c_int;
    p.cast::<c_void>()
}

/// Copy string `s` into arena memory. Mirrors C `getroom_save`.
///
/// # Safety
/// `spin` and `s` must be valid non-null pointers. `s` must be NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn rs_getroom_save(spin: *mut SpellinfoT, s: *mut c_char) -> *mut c_char {
    let s_size = libc::strlen(s.cast::<libc::c_char>()) + 1;
    let dest = rs_getroom(spin, s_size, false).cast::<c_char>();
    libc::memcpy(dest.cast::<c_void>(), s.cast::<c_void>(), s_size);
    dest
}

/// Free the sblock_T linked list. Mirrors C `free_blocks`.
///
/// # Safety
/// `bl` must be a valid pointer to a `sblock_T` linked list, or NULL.
#[export_name = "free_blocks"]
pub unsafe extern "C" fn rs_free_blocks(mut bl: *mut SblockT) {
    while !bl.is_null() {
        let next = (*bl).sb_next;
        xfree(bl.cast::<c_void>());
        bl = next;
    }
}

/// Add a REP/SAL from-to item to `gap`. Case-folds both strings.
/// Mirrors C `add_fromto`.
///
/// # Safety
/// `spin`, `gap`, `from`, and `to` must be valid non-null pointers.
/// `from` and `to` must be NUL-terminated C strings.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn rs_add_fromto(
    spin: *mut SpellinfoT,
    gap: *mut crate::GArrayRaw,
    from: *const c_char,
    to: *const c_char,
) {
    const MAXWLEN: usize = 254;
    let mut word_buf = [0u8; MAXWLEN + 1];

    // Grow the array by 1, then get pointer to the new (last) element.
    ga_grow(gap, 1);
    let ftp = ((*gap).ga_data.cast::<FromtoC>()).add((*gap).ga_len as usize);
    (*gap).ga_len += 1;

    // Case-fold "from" string.
    let from_len = libc::strlen(from.cast::<libc::c_char>()) as c_int;
    spell_casefold(
        curwin_spell,
        from,
        from_len,
        word_buf.as_mut_ptr().cast::<c_char>(),
        (MAXWLEN + 1) as c_int,
    );
    (*ftp).ft_from = rs_getroom_save(spin, word_buf.as_ptr().cast::<c_char>().cast_mut());

    // Case-fold "to" string.
    let to_len = libc::strlen(to.cast::<libc::c_char>()) as c_int;
    spell_casefold(
        curwin_spell,
        to,
        to_len,
        word_buf.as_mut_ptr().cast::<c_char>(),
        (MAXWLEN + 1) as c_int,
    );
    (*ftp).ft_to = rs_getroom_save(spin, word_buf.as_ptr().cast::<c_char>().cast_mut());
}

// =============================================================================
// Phase 3: process_compflags, spell_free_aff, store_aff_word
// =============================================================================

// Phase 3 uses: vim_regfree_spell, vim_regexec_prog, mb_charlen, utfc_ptr2len_spell,
// utf_head_off, xstrlcpy, hash_clear, hash_add -- all declared in the main extern block above.

/// Process the "compflags" string and append compound IDs to spin->si_compflags.
/// Mirrors C `process_compflags`.
///
/// # Safety
/// `spin`, `aff`, and `compflags` must be valid non-null pointers.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_ptr_alignment
)]
pub unsafe extern "C" fn rs_process_compflags(
    spin: *mut SpellinfoT,
    aff: *mut AfffileT,
    compflags: *mut c_char,
) {
    // Make room for old and new compflags concatenated with '/'.
    let new_len = libc::strlen(compflags.cast::<libc::c_char>()) as c_int + 1;
    let total_len = if (*spin).si_compflags.is_null() {
        new_len
    } else {
        new_len + libc::strlen((*spin).si_compflags.cast::<libc::c_char>()) as c_int + 1
    };
    let p = rs_getroom(spin, total_len as usize, false).cast::<c_char>();
    if !(*spin).si_compflags.is_null() {
        xstrlcpy(p, (*spin).si_compflags, total_len as usize);
        // Append "/"
        let plen = libc::strlen(p.cast::<libc::c_char>());
        *p.add(plen) = b'/' as c_char;
        *p.add(plen + 1) = 0;
    }
    (*spin).si_compflags = p;
    // tp points to end of current string
    let mut tp = p.add(libc::strlen(p.cast::<libc::c_char>())).cast::<u8>();

    let ft = (*aff).af_flagtype;
    let mut scan = compflags;
    while *scan != 0 {
        if crate::vim_strchr(c"/?*+[]".as_ptr(), i32::from(*scan as u8)).is_null() {
            // Flag character: parse and map to ID.
            let prevp = scan;
            let flag = get_affitem_inner(ft, &raw mut scan);
            if flag != 0 {
                // Build key for hashtable lookup.
                let mut key = [0u8; AH_KEY_LEN];
                let key_len = (scan.offset_from(prevp) as usize).min(AH_KEY_LEN - 1);
                std::ptr::copy_nonoverlapping(prevp.cast::<u8>(), key.as_mut_ptr(), key_len);
                key[key_len] = 0;

                let hi = hash_find(&raw const (*aff).af_comp, key.as_ptr().cast::<c_char>());
                let id: c_int;
                if !hi.is_null() && !hi_is_empty(hi) {
                    #[allow(clippy::cast_ptr_alignment)]
                    let ci = (*hi).hi_key.cast::<CompitemT>();
                    id = (*ci).ci_newID;
                } else {
                    // New compound item.
                    let ci = rs_getroom(spin, std::mem::size_of::<CompitemT>(), true)
                        .cast::<CompitemT>();
                    // Zero already done by getroom (xcalloc). Set fields.
                    std::ptr::copy_nonoverlapping(
                        key.as_ptr().cast::<c_char>(),
                        (*ci).ci_key.as_mut_ptr(),
                        AH_KEY_LEN,
                    );
                    (*ci).ci_flag = flag;
                    // Pick an ID not used as regexp special char.
                    let mut new_id: c_int;
                    loop {
                        rs_check_renumber(spin);
                        (*spin).si_newcompID -= 1;
                        new_id = (*spin).si_newcompID;
                        if crate::vim_strchr(c"/?*+[]\\-^".as_ptr(), new_id).is_null() {
                            break;
                        }
                    }
                    (*ci).ci_newID = new_id;
                    id = new_id;
                    hash_add(&raw mut (*aff).af_comp, (*ci).ci_key.as_mut_ptr());
                }
                *tp = id as u8;
                tp = tp.add(1);
            }
            if ft == AFT_NUM && *scan == b',' as c_char {
                scan = scan.add(1);
            }
        } else {
            // Non-flag regexp char (/?*+[] etc): copy directly.
            *tp = *scan as u8;
            tp = tp.add(1);
            scan = scan.add(1);
        }
    }
    *tp = 0;
}

/// Free the afffile_T structure (ae_prog entries + hashtables).
/// Mirrors C `spell_free_aff`.
///
/// # Safety
/// `aff` must be a valid non-null pointer to an `afffile_T`.
#[no_mangle]
#[allow(clippy::cast_ptr_alignment, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_spell_free_aff(aff: *mut AfffileT) {
    xfree((*aff).af_enc.cast::<c_void>());

    // Free ae_prog for all entries in af_pref and af_suff.
    for which in 0..2usize {
        let ht = if which == 0 {
            &raw mut (*aff).af_pref
        } else {
            &raw mut (*aff).af_suff
        };
        let mut todo = (*ht).ht_used as c_int;
        let mut hi = (*ht).ht_array;
        while todo > 0 {
            if !hi_is_empty(hi) {
                todo -= 1;
                let ah = (*hi).hi_key.cast::<AffheaderT>();
                let mut ae = (*ah).ah_first;
                while !ae.is_null() {
                    if !(*ae).ae_prog.is_null() {
                        vim_regfree_spell((*ae).ae_prog);
                    }
                    ae = (*ae).ae_next;
                }
            }
            hi = hi.add(1);
        }
    }

    hash_clear(&raw mut (*aff).af_pref);
    hash_clear(&raw mut (*aff).af_suff);
    hash_clear(&raw mut (*aff).af_comp);
}

// Flags for store_aff_word's condit parameter (must match C defines in spellfile.c).
const CONDIT_COMB: c_int = 1; // affix must combine
const CONDIT_CFIX: c_int = 2; // affix must have CIRCUMFIX flag
const CONDIT_SUF: c_int = 4; // add a suffix for matching flags
const CONDIT_AFF: c_int = 8; // word already has an affix

// WF_ word flags used in store_aff_word (from spell_defs.h).
const WF_HAS_AFF: c_int = 0x0100;
const WF_NOCOMPBEF: c_int = 0x1000;
const WF_NOCOMPAFT: c_int = 0x2000;

/// Apply affixes to a word and store the resulting words.
/// Mirrors C `store_aff_word`. Recursive.
///
/// # Safety
/// All pointers must be valid. `spin`, `word`, `afflist`, `affile`, `ht` must be non-null.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_ptr_alignment,
    clippy::too_many_arguments,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_store_aff_word(
    spin: *mut SpellinfoT,
    word: *const c_char,
    afflist: *mut c_char,
    affile: *mut AfffileT,
    ht: *mut crate::HashtabRaw,
    xht: *mut crate::HashtabRaw,
    condit: c_int,
    flags: c_int,
    pfxlist: *mut c_char,
    pfxlen: c_int,
) -> c_int {
    const MAXWLEN: usize = 254;
    let mut retval: c_int = 0; // OK

    let wordlen = libc::strlen(word.cast::<libc::c_char>());
    let ft = (*affile).af_flagtype;

    let mut todo = (*ht).ht_used as c_int;
    let mut hi = (*ht).ht_array;

    while todo > 0 && retval == 0 {
        if !hi_is_empty(hi) {
            todo -= 1;
            let ah = (*hi).hi_key.cast::<AffheaderT>();

            // Check that the affix combines (if required) and that the word supports it.
            if ((condit & CONDIT_COMB) == 0 || (*ah).ah_combine != 0)
                && flag_in_afflist_inner(ft, afflist, (*ah).ah_flag)
            {
                let mut ae = (*ah).ah_first;
                while !ae.is_null() {
                    // Check conditions for applying this affix entry.
                    let chop_len = if (*ae).ae_chop.is_null() {
                        0usize
                    } else {
                        libc::strlen((*ae).ae_chop.cast::<libc::c_char>())
                    };
                    let prog_matches = if (*ae).ae_prog.is_null() {
                        true
                    } else {
                        vim_regexec_prog(&raw mut (*ae).ae_prog, false, word, 0)
                    };
                    let circumfix_ok = ((condit & CONDIT_CFIX) == 0)
                        == ((condit & CONDIT_AFF) == 0
                            || (*ae).ae_flags.is_null()
                            || !flag_in_afflist_inner(ft, (*ae).ae_flags, (*affile).af_circumfix));

                    if (!xht.is_null()
                        || (*affile).af_pfxpostpone == 0
                        || !(*ae).ae_chop.is_null()
                        || !(*ae).ae_flags.is_null())
                        && chop_len < wordlen
                        && prog_matches
                        && circumfix_ok
                    {
                        // Build new word: apply affix.
                        let mut newword_buf = [0u8; MAXWLEN + 1];
                        let newword = newword_buf.as_mut_ptr().cast::<c_char>();

                        if xht.is_null() {
                            // prefix: chop/add at start of word
                            if (*ae).ae_add.is_null() {
                                *newword = 0;
                            } else {
                                xstrlcpy(newword, (*ae).ae_add, MAXWLEN + 1);
                            }
                            // Skip chop string in word.
                            let mut p = word.cast_mut();
                            if !(*ae).ae_chop.is_null() {
                                let mut i = mb_charlen((*ae).ae_chop);
                                while i > 0 {
                                    let step = utfc_ptr2len_spell(p) as usize;
                                    p = p.add(step);
                                    i -= 1;
                                }
                            }
                            // Append rest of word after affix.
                            let nlen = libc::strlen(newword.cast::<libc::c_char>());
                            libc::strncat(newword, p, MAXWLEN - nlen);
                        } else {
                            // suffix: chop/add at end of word
                            xstrlcpy(newword, word, MAXWLEN + 1);
                            if !(*ae).ae_chop.is_null() {
                                // Remove chop from end of word.
                                let nlen = libc::strlen(newword.cast::<libc::c_char>());
                                let mut p = newword.add(nlen);
                                let mut i = mb_charlen((*ae).ae_chop);
                                while i > 0 {
                                    // Back up one character.
                                    let off = utf_head_off(newword, p.sub(1)) as usize;
                                    p = p.sub(off + 1);
                                    i -= 1;
                                }
                                *p = 0;
                            }
                            if !(*ae).ae_add.is_null() {
                                let nlen = libc::strlen(newword.cast::<libc::c_char>());
                                libc::strncat(newword, (*ae).ae_add, MAXWLEN - nlen);
                            }
                        }

                        let mut use_flags = flags;
                        let mut use_pfxlist = pfxlist;
                        let mut use_pfxlen = pfxlen;
                        let mut need_affix = false;
                        let mut use_condit = condit | CONDIT_COMB | CONDIT_AFF;

                        let mut store_afflist = [0u8; MAXWLEN + 1];
                        let mut pfx_pfxlist = [0u8; MAXWLEN + 1];

                        if !(*ae).ae_flags.is_null() {
                            use_flags |= rs_get_affix_flags(affile, (*ae).ae_flags);

                            if (*affile).af_needaffix != 0
                                && flag_in_afflist_inner(ft, (*ae).ae_flags, (*affile).af_needaffix)
                            {
                                need_affix = true;
                            }

                            if (*affile).af_circumfix != 0
                                && flag_in_afflist_inner(ft, (*ae).ae_flags, (*affile).af_circumfix)
                            {
                                use_condit |= CONDIT_CFIX;
                                if (condit & CONDIT_CFIX) == 0 {
                                    need_affix = true;
                                }
                            }

                            if (*affile).af_pfxpostpone != 0 || !(*spin).si_compflags.is_null() {
                                let sap = store_afflist.as_mut_ptr().cast::<c_char>();
                                if (*affile).af_pfxpostpone != 0 {
                                    use_pfxlen = rs_get_pfxlist(affile, (*ae).ae_flags, sap);
                                } else {
                                    use_pfxlen = 0;
                                }
                                use_pfxlist = sap;

                                // Combine prefix IDs (avoid duplicates).
                                let mut i = 0;
                                while i < pfxlen {
                                    let mut j = 0;
                                    while j < use_pfxlen {
                                        if *pfxlist.add(i as usize) == *use_pfxlist.add(j as usize)
                                        {
                                            break;
                                        }
                                        j += 1;
                                    }
                                    if j == use_pfxlen {
                                        *use_pfxlist.add(use_pfxlen as usize) =
                                            *pfxlist.add(i as usize);
                                        use_pfxlen += 1;
                                    }
                                    i += 1;
                                }

                                if (*spin).si_compflags.is_null() {
                                    *use_pfxlist.add(use_pfxlen as usize) = 0;
                                } else {
                                    rs_get_compflags(
                                        affile,
                                        (*ae).ae_flags,
                                        use_pfxlist.add(use_pfxlen as usize),
                                    );
                                }

                                // Combine compound flags (avoid duplicates).
                                let mut i = pfxlen;
                                while *pfxlist.add(i as usize) != 0 {
                                    let mut j = use_pfxlen;
                                    while *use_pfxlist.add(j as usize) != 0 {
                                        if *pfxlist.add(i as usize) == *use_pfxlist.add(j as usize)
                                        {
                                            break;
                                        }
                                        j += 1;
                                    }
                                    if *use_pfxlist.add(j as usize) == 0 {
                                        *use_pfxlist.add(j as usize) = *pfxlist.add(i as usize);
                                        *use_pfxlist.add(j as usize + 1) = 0;
                                    }
                                    i += 1;
                                }
                            }
                        }

                        // COMPOUNDFORBIDFLAG: copy pfxlist to prevent modification.
                        if !use_pfxlist.is_null() && (*ae).ae_compforbid != 0 {
                            libc::memcpy(
                                pfx_pfxlist.as_mut_ptr().cast::<c_void>(),
                                use_pfxlist.cast::<c_void>(),
                                use_pfxlen as usize,
                            );
                            use_pfxlist = pfx_pfxlist.as_mut_ptr().cast::<c_char>();
                        }

                        // Postponed prefixes.
                        if !(*spin).si_prefroot.is_null()
                            && !(*(*spin).si_prefroot).wn_sibling.is_null()
                        {
                            use_flags |= WF_HAS_AFF;
                            if (*ah).ah_combine == 0 && !use_pfxlist.is_null() {
                                use_pfxlist = use_pfxlist.add(use_pfxlen as usize);
                            }
                        }

                        // Compounding: forbid on the side where affix is applied.
                        if !(*spin).si_compflags.is_null() && (*ae).ae_comppermit == 0 {
                            if xht.is_null() {
                                use_flags |= WF_NOCOMPBEF;
                            } else {
                                use_flags |= WF_NOCOMPAFT;
                            }
                        }

                        // Store the modified word.
                        if rs_store_word(
                            spin,
                            newword,
                            use_flags,
                            (*spin).si_region,
                            use_pfxlist,
                            need_affix,
                        ) != 0
                        {
                            retval = 1; // FAIL
                        }

                        // Recurse: add suffix after prefix/first-suffix.
                        if (condit & CONDIT_SUF) != 0 && !(*ae).ae_flags.is_null() {
                            let sub_condit =
                                use_condit & (if xht.is_null() { !0 } else { !CONDIT_SUF });
                            if rs_store_aff_word(
                                spin,
                                newword,
                                (*ae).ae_flags,
                                affile,
                                &raw mut (*affile).af_suff,
                                xht,
                                sub_condit,
                                use_flags,
                                use_pfxlist,
                                pfxlen,
                            ) != 0
                            {
                                retval = 1;
                            }
                        }

                        // Recurse: add prefix after suffix (combining).
                        if !xht.is_null() && (*ah).ah_combine != 0 {
                            if rs_store_aff_word(
                                spin,
                                newword,
                                afflist,
                                affile,
                                xht,
                                std::ptr::null_mut(),
                                use_condit,
                                use_flags,
                                use_pfxlist,
                                pfxlen,
                            ) != 0
                            {
                                retval = 1;
                            }
                            if !(*ae).ae_flags.is_null()
                                && rs_store_aff_word(
                                    spin,
                                    newword,
                                    (*ae).ae_flags,
                                    affile,
                                    xht,
                                    std::ptr::null_mut(),
                                    use_condit,
                                    use_flags,
                                    use_pfxlist,
                                    pfxlen,
                                ) != 0
                            {
                                retval = 1;
                            }
                        }
                    }

                    ae = (*ae).ae_next;
                }
            }
        }
        hi = hi.add(1);
    }

    retval
}

// =============================================================================
// Phase 4: File readers (spell_read_dic, spell_read_wordfile)
// =============================================================================

/// Repr(C) mirror of vimconv_T (mbyte_defs.h).
/// sizeof = 24, alignof = 8.
#[repr(C)]
struct VimconvT {
    pub vc_type: c_int, // ConvFlags: 0=CONV_NONE
    pub vc_factor: c_int,
    pub vc_fd: u64, // iconv_t (8 bytes on 64-bit)
    pub vc_fail: bool,
    _pad: [u8; 7],
}

const _: () = {
    assert!(std::mem::size_of::<VimconvT>() == 24);
    assert!(std::mem::offset_of!(VimconvT, vc_type) == 0);
    assert!(std::mem::offset_of!(VimconvT, vc_fd) == 8);
    assert!(std::mem::offset_of!(VimconvT, vc_fail) == 16);
};

extern "C" {
    // os_fopen: file open. Use a new alias to avoid conflict with os_fopen_sug.
    #[link_name = "os_fopen"]
    fn os_fopen_ph4(path: *const c_char, flags: *const c_char) -> *mut libc::FILE;
    fn vim_fgets(buf: *mut c_char, size: c_int, fp: *mut libc::FILE) -> bool;
    fn string_convert(vcp: *const VimconvT, ptr: *mut c_char, lenp: *mut usize) -> *mut c_char;
    fn convert_setup(vcp: *mut VimconvT, from: *mut c_char, to: *mut c_char) -> c_int;
    fn enc_canonize(enc: *mut c_char) -> *mut c_char;
    fn has_non_ascii(s: *const c_char) -> bool;
    fn msg_outtrans_long(longstr: *const c_char, hl_id: c_int);
    fn os_time() -> i64;
    fn vim_snprintf(str_: *mut c_char, str_m: usize, fmt: *const c_char, ...) -> c_int;
    #[link_name = "got_int"]
    static got_int_global: bool;
    #[link_name = "IObuff"]
    static mut IObuff_global: [c_char; 1025];
    #[link_name = "p_verbose"]
    static p_verbose_global: i64;
    #[link_name = "p_enc"]
    static p_enc_global: *mut c_char;
    #[link_name = "msg_didout"]
    static mut msg_didout_global: bool;
    #[link_name = "msg_col"]
    static mut msg_col_global: c_int;
}

/// C OK/FAIL constants.
const OK: c_int = 0;
const FAIL: c_int = 1;

const CONV_NONE: c_int = 0;

const MAXREGIONS: usize = 8;

/// Get the `VimconvT` pointer embedded in `spellinfo_T.si_conv` (offset 112).
///
/// # Safety
/// `spin` must be a valid non-null pointer.
#[inline]
#[allow(clippy::cast_ptr_alignment)]
unsafe fn spin_conv(spin: *mut SpellinfoT) -> *mut VimconvT {
    (spin as *mut u8).add(112).cast::<VimconvT>()
}

/// Read a .dic dictionary file and store all words.
/// Mirrors C `spell_read_dic`.
///
/// # Safety
/// `spin`, `fname`, and `affile` must be valid non-null pointers.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::too_many_lines,
    clippy::if_not_else,
    clippy::unnecessary_cast
)]
pub unsafe extern "C" fn rs_spell_read_dic(
    spin: *mut SpellinfoT,
    fname: *const c_char,
    affile: *mut AfffileT,
) -> c_int {
    const MAXWLEN: usize = 254;
    let mut retval = OK;

    let fd = os_fopen_ph4(fname, c"r".as_ptr());
    if fd.is_null() {
        semsg_sug(c"E484: Can't open file %s".as_ptr(), fname);
        return FAIL;
    }

    // The hashtable detects duplicate words.
    let mut ht = std::mem::zeroed::<crate::HashtabRaw>();
    hash_init(&raw mut ht);

    // Show progress message.
    let iobuff = std::ptr::addr_of_mut!(IObuff_global).cast::<c_char>();
    vim_snprintf(
        iobuff,
        1025,
        c"Reading dictionary file %s...".as_ptr(),
        fname,
    );
    rs_spell_message(spin, iobuff);

    // Start with a large msg_count so first line triggers message.
    (*spin).si_msg_count = 999_999;

    // Read and ignore the first line (word count).
    {
        let mut line_buf = [0i8; 4096];
        if !vim_fgets(line_buf.as_mut_ptr(), 4096, fd) {
            // Check that first line is a number (word count header).
            let mut p = line_buf.as_mut_ptr();
            while *p == b' ' as i8 || *p == b'\t' as i8 {
                p = p.add(1);
            }
            if ascii_isdigit(i32::from(*p as u8)) == 0 {
                semsg_sug(c"E760: No word count in %s".as_ptr(), fname);
            }
        }
    }

    let mut line_buf = [0i8; 4096usize];
    let mut word_buf = [0u8; 4096usize];
    let mut store_afflist = [0u8; MAXWLEN + 1];
    let mut lnum: c_int = 1;
    let mut non_ascii: c_int = 0;
    let mut duplicate: c_int = 0;
    let mut last_msg_time: i64 = 0;

    while !vim_fgets(line_buf.as_mut_ptr(), 4096, fd) && !std::ptr::addr_of!(got_int_global).read()
    {
        line_breakcheck_sug();
        lnum += 1;

        // Convert encoding if needed.
        let conv = spin_conv(spin);
        let pc: *mut c_char = if (*conv).vc_type != CONV_NONE {
            let p = string_convert(conv, line_buf.as_mut_ptr(), std::ptr::null_mut());
            if p.is_null() {
                semsg_sug(
                    c"Conversion failure for word in %s line %d: %s".as_ptr(),
                    fname,
                    lnum,
                    line_buf.as_ptr(),
                );
                continue;
            }
            p
        } else {
            std::ptr::null_mut()
        };

        let src_line: *const c_char = if pc.is_null() { line_buf.as_ptr() } else { pc };
        let src_len = libc::strlen(src_line.cast::<libc::c_char>());

        // Parse the dic line via Rust.
        let mut rs_res = DicLineResult {
            word_len: 0,
            affix_offset: 0xFFFF,
            affix_len: 0,
        };
        let parse_ret = rs_parse_dic_line(
            src_line.cast::<u8>(),
            src_len,
            word_buf.as_mut_ptr(),
            word_buf.len(),
            &raw mut rs_res,
        );
        if parse_ret != 0 {
            xfree(pc.cast::<c_void>());
            continue;
        }

        word_buf[rs_res.word_len as usize] = 0;
        let w = word_buf.as_mut_ptr().cast::<c_char>();

        // Get affix list pointer (into the original src_line buffer).
        let afflist: *mut c_char = if rs_res.affix_offset == 0xFFFF {
            std::ptr::null_mut()
        } else {
            src_line.add(rs_res.affix_offset as usize).cast_mut()
        };

        // Skip non-ASCII words when si_ascii is set.
        if (*spin).si_ascii != 0 && has_non_ascii(w) {
            non_ascii += 1;
            xfree(pc.cast::<c_void>());
            continue;
        }

        // Verbose progress message every 10000 words but at most once per second.
        if (*spin).si_verbose != 0 && (*spin).si_msg_count > 10000 {
            (*spin).si_msg_count = 0;
            let now = os_time();
            if now > last_msg_time {
                last_msg_time = now;
                let mut message = [0i8; 4096usize + MAXWLEN];
                vim_snprintf(
                    message.as_mut_ptr(),
                    message.len(),
                    c"line %6d, word %6d - %s".as_ptr(),
                    lnum,
                    (*spin).si_foldwcount + (*spin).si_keepwcount,
                    w,
                );
                msg_start();
                msg_outtrans_long(message.as_ptr(), 0);
                msg_clr_eos();
                std::ptr::addr_of_mut!(msg_didout_global).write(false);
                std::ptr::addr_of_mut!(msg_col_global).write(0);
                ui_flush();
            }
        }

        // Store the word in the hashtable to detect duplicates.
        let dw = rs_getroom_save(spin, w);
        if dw.is_null() {
            retval = FAIL;
            xfree(pc.cast::<c_void>());
            break;
        }

        let hash = hash_hash(dw);
        let dw_len = libc::strlen(dw.cast::<libc::c_char>());
        let hi = hash_lookup(&raw const ht, dw, dw_len, hash);
        if hi_is_empty(hi) {
            hash_add_item(&raw mut ht, hi, dw, hash);
        } else {
            if std::ptr::addr_of!(p_verbose_global).read() > 0 {
                semsg_sug(
                    c"Duplicate word in %s line %d: %s".as_ptr(),
                    fname,
                    lnum,
                    dw,
                );
            } else if duplicate == 0 {
                semsg_sug(
                    c"First duplicate word in %s line %d: %s".as_ptr(),
                    fname,
                    lnum,
                    dw,
                );
            }
            duplicate += 1;
        }

        let mut flags: c_int = 0;
        store_afflist[0] = 0;
        let mut pfxlen: c_int = 0;
        let mut need_affix = false;

        if !afflist.is_null() {
            flags |= rs_get_affix_flags(affile, afflist);

            if (*affile).af_needaffix != 0
                && flag_in_afflist_inner((*affile).af_flagtype, afflist, (*affile).af_needaffix)
            {
                need_affix = true;
            }

            if (*affile).af_pfxpostpone != 0 {
                pfxlen =
                    rs_get_pfxlist(affile, afflist, store_afflist.as_mut_ptr().cast::<c_char>());
            }

            if !(*spin).si_compflags.is_null() {
                rs_get_compflags(
                    affile,
                    afflist,
                    store_afflist
                        .as_mut_ptr()
                        .add(pfxlen as usize)
                        .cast::<c_char>(),
                );
            }
        }

        // Add the word to the word tree(s).
        if rs_store_word(
            spin,
            dw,
            flags,
            (*spin).si_region,
            store_afflist.as_ptr().cast::<c_char>(),
            need_affix,
        ) != OK
        {
            retval = FAIL;
        }

        if !afflist.is_null() {
            // Find all matching suffixes.
            if rs_store_aff_word(
                spin,
                dw,
                afflist,
                affile,
                &raw mut (*affile).af_suff,
                &raw mut (*affile).af_pref,
                CONDIT_SUF,
                flags,
                store_afflist.as_mut_ptr().cast::<c_char>(),
                pfxlen,
            ) != OK
            {
                retval = FAIL;
            }

            // Find all matching prefixes.
            if rs_store_aff_word(
                spin,
                dw,
                afflist,
                affile,
                &raw mut (*affile).af_pref,
                std::ptr::null_mut(),
                CONDIT_SUF,
                flags,
                store_afflist.as_mut_ptr().cast::<c_char>(),
                pfxlen,
            ) != OK
            {
                retval = FAIL;
            }
        }

        xfree(pc.cast::<c_void>());
    }

    if duplicate > 0 {
        semsg_sug(c"%d duplicate word(s) in %s".as_ptr(), duplicate, fname);
    }
    if (*spin).si_ascii != 0 && non_ascii > 0 {
        semsg_sug(
            c"Ignored %d word(s) with non-ASCII characters in %s".as_ptr(),
            non_ascii,
            fname,
        );
    }
    hash_clear(&raw mut ht);

    libc::fclose(fd);
    retval
}

/// Read a plain word list file and store all words.
/// Mirrors C `spell_read_wordfile`.
///
/// # Safety
/// `spin` and `fname` must be valid non-null pointers.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::too_many_lines,
    clippy::if_not_else,
    clippy::unnecessary_cast
)]
pub unsafe extern "C" fn rs_spell_read_wordfile(
    spin: *mut SpellinfoT,
    fname: *const c_char,
) -> c_int {
    let mut retval = OK;
    let mut lnum: c_int = 0;
    let mut non_ascii: c_int = 0;
    let mut did_word = false;

    let fd = os_fopen_ph4(fname, c"r".as_ptr());
    if fd.is_null() {
        semsg_sug(c"E484: Can't open file %s".as_ptr(), fname);
        return FAIL;
    }

    let iobuff = std::ptr::addr_of_mut!(IObuff_global).cast::<c_char>();
    vim_snprintf(iobuff, 1025, c"Reading word file %s...".as_ptr(), fname);
    rs_spell_message(spin, iobuff);

    let mut rline = [0i8; 4096usize];
    let mut pc: *mut c_char = std::ptr::null_mut();

    while !vim_fgets(rline.as_mut_ptr(), 4096, fd) && !std::ptr::addr_of!(got_int_global).read() {
        line_breakcheck_sug();
        lnum += 1;

        // Convert encoding if needed.
        xfree(pc.cast::<c_void>());
        pc = std::ptr::null_mut();
        let conv = spin_conv(spin);
        if (*conv).vc_type != CONV_NONE {
            pc = string_convert(conv, rline.as_mut_ptr(), std::ptr::null_mut());
            if pc.is_null() {
                semsg_sug(
                    c"Conversion failure for word in %s line %d: %s".as_ptr(),
                    fname,
                    lnum,
                    rline.as_ptr(),
                );
                continue;
            }
        }

        let line: *const c_char = if pc.is_null() { rline.as_ptr() } else { pc };
        let line_len = libc::strlen(line.cast::<libc::c_char>());

        let mut rs_res = WordfileLineResult {
            directive: [0u8; 16],
            word_len: 0,
            word_end_offset: 0,
            flags: 0,
            regionmask: 0,
            region_count: 0,
        };
        let parse_ret = rs_parse_wordfile_line(
            line.cast::<u8>(),
            line_len,
            (*spin).si_region_count,
            &raw mut rs_res,
        );

        if parse_ret == 2 {
            continue;
        }

        if parse_ret == 1 {
            // Directive line.
            if rs_res.directive[0] == b'e' {
                // /encoding= directive
                if (*conv).vc_type != CONV_NONE {
                    semsg_sug(
                        c"Duplicate /encoding= line ignored in %s line %d: %s".as_ptr(),
                        fname,
                        lnum,
                        line,
                    );
                } else if did_word {
                    semsg_sug(
                        c"/encoding= line after word ignored in %s line %d: %s".as_ptr(),
                        fname,
                        lnum,
                        line,
                    );
                } else {
                    let enc_val = line.add(rs_res.word_end_offset as usize).cast_mut();
                    let enc = enc_canonize(enc_val);
                    let p_enc = std::ptr::addr_of!(p_enc_global).read();
                    if (*spin).si_ascii == 0 && convert_setup(conv, enc, p_enc) == FAIL {
                        semsg_sug(
                            c"Conversion in %s not supported: from %s to %s".as_ptr(),
                            fname,
                            enc_val,
                            p_enc,
                        );
                    }
                    xfree(enc.cast::<c_void>());
                    (*conv).vc_fail = true;
                }
            } else if rs_res.directive[0] == b'r' {
                // /regions= directive
                if (*spin).si_region_count > 1 {
                    semsg_sug(
                        c"Duplicate /regions= line ignored in %s line %d: %s".as_ptr(),
                        fname,
                        lnum,
                        line.add(1),
                    );
                } else {
                    let reg_val = line.add(rs_res.word_end_offset as usize);
                    if (rs_res.word_len as usize) > MAXREGIONS * 2 {
                        semsg_sug(
                            c"Too many regions in %s line %d: %s".as_ptr(),
                            fname,
                            lnum,
                            reg_val,
                        );
                    } else {
                        (*spin).si_region_count = i32::from(rs_res.region_count);
                        libc::strncpy(
                            (*spin).si_region_name.as_mut_ptr(),
                            reg_val,
                            (*spin).si_region_name.len() - 1,
                        );
                        (*spin).si_region = (1 << (*spin).si_region_count) - 1;
                    }
                }
            } else {
                semsg_sug(
                    c"/ line ignored in %s line %d: %s".as_ptr(),
                    fname,
                    lnum,
                    line,
                );
            }
            continue;
        }

        if parse_ret == 3 {
            semsg_sug(
                c"Invalid region nr in %s line %d: %s".as_ptr(),
                fname,
                lnum,
                line,
            );
            continue;
        }

        if parse_ret == 4 {
            semsg_sug(
                c"Unrecognized flags in %s line %d: %s".as_ptr(),
                fname,
                lnum,
                line,
            );
            continue;
        }

        if parse_ret != 0 {
            continue;
        }

        // Ordinary word line.
        let wlen = (rs_res.word_len as usize).min(4095);
        let mut word_copy = [0i8; 4096usize];
        libc::memcpy(
            word_copy.as_mut_ptr().cast::<c_void>(),
            line.cast::<c_void>(),
            wlen,
        );
        word_copy[wlen] = 0;

        let word_flags = rs_res.flags;
        let regionmask = if (word_flags & wordfile_flags::WF_REGION) != 0 {
            rs_res.regionmask
        } else {
            (*spin).si_region
        };

        // Skip non-ASCII words when si_ascii is set.
        if (*spin).si_ascii != 0 && has_non_ascii(word_copy.as_ptr()) {
            non_ascii += 1;
            continue;
        }

        // Store the word.
        if rs_store_word(
            spin,
            word_copy.as_ptr(),
            word_flags,
            regionmask,
            std::ptr::null(),
            false,
        ) != OK
        {
            retval = FAIL;
            break;
        }
        did_word = true;
    }

    xfree(pc.cast::<c_void>());
    libc::fclose(fd);

    if (*spin).si_ascii != 0 && non_ascii > 0 {
        let iobuff2 = std::ptr::addr_of_mut!(IObuff_global).cast::<c_char>();
        vim_snprintf(
            iobuff2,
            1025,
            c"Ignored %d words with non-ASCII characters".as_ptr(),
            non_ascii,
        );
        rs_spell_message(spin, iobuff2);
    }

    retval
}

// =============================================================================
// Phase 4 (cont.): spell_read_aff
// =============================================================================

// COMP_ flags for compoptions (match spell_defs.h).
const COMP_CHECKDUP: c_int = 1;
const COMP_CHECKREP: c_int = 2;
const COMP_CHECKCASE: c_int = 4;
const COMP_CHECKTRIPLE: c_int = 8;

// WFP_ prefix flags (match spell_defs.h).
const WFP_NC: c_int = 0x02;
const WFP_UP: c_int = 0x04;
const WFP_COMPPERMIT: c_int = 0x08;
const WFP_COMPFORBID: c_int = 0x10;

// vim_regcomp flags (match regexp_defs.h).
const RE_MAGIC: c_int = 1;
const RE_STRING: c_int = 2;
const RE_STRICT: c_int = 16;

extern "C" {
    fn skipdigits(q: *const c_char) -> *const c_char;
    fn utf_ptr2char(p: *const c_char) -> c_int;
    #[link_name = "nvim_spell_toupper"]
    fn spell_toupper(c: c_int) -> c_int;
    fn onecap_copy(word: *const c_char, wcopy: *mut c_char, upper: bool);
    fn init_spell_chartab();
    fn ga_concat(gap: *mut crate::GArrayRaw, s: *const c_char);
    fn ga_append(gap: *mut crate::GArrayRaw, c: c_char);
    #[link_name = "smsg"]
    fn smsg_aff(hl_id: c_int, fmt: *const c_char, ...) -> c_int;
    #[link_name = "semsg"]
    fn semsg_aff(fmt: *const c_char, ...) -> bool;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    #[link_name = "xfree"]
    fn xfree_aff(ptr: *mut c_void);
    #[link_name = "vim_strchr"]
    fn vim_strchr_aff(s: *const c_char, c: c_int) -> *mut c_char;
}

/// Reads an affix file "fname".  Returns an `AfffileT` pointer, NULL for failure.
/// Mirrors C `spell_read_aff`.
///
/// # Safety
/// `spin` and `fname` must be valid non-null pointers.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_ptr_alignment,
    clippy::too_many_lines,
    clippy::if_not_else,
    clippy::unnecessary_cast,
    clippy::cognitive_complexity
)]
pub unsafe extern "C" fn rs_spell_read_aff(
    spin: *mut SpellinfoT,
    fname: *mut c_char,
) -> *mut AfffileT {
    const MAXITEMCNT: usize = 30;
    const MAXLINELEN_AFF: usize = 500;

    // Open file.
    let fd = os_fopen_ph4(fname, c"r".as_ptr());
    if fd.is_null() {
        semsg_aff(c"%s".as_ptr(), fname); // e_notopen equivalent
        return std::ptr::null_mut();
    }

    let iobuff = std::ptr::addr_of_mut!(IObuff_global).cast::<c_char>();
    vim_snprintf(iobuff, 1025, c"Reading affix file %s...".as_ptr(), fname);
    rs_spell_message(spin, iobuff);

    // Only do REP/REPSAL/SAL/MAP lines when not done in another .aff already.
    let do_rep = (*spin).si_rep.ga_len == 0;
    let do_repsal = (*spin).si_repsal.ga_len == 0;
    let do_sal = (*spin).si_sal.ga_len == 0;
    let do_mapline = (*spin).si_map.ga_len == 0;

    // Allocate and init the afffile_T.
    let aff = rs_getroom(spin, std::mem::size_of::<AfffileT>(), true).cast::<AfffileT>();
    hash_init(&raw mut (*aff).af_pref);
    hash_init(&raw mut (*aff).af_suff);
    hash_init(&raw mut (*aff).af_comp);

    let mut rline = [0u8; MAXLINELEN_AFF];
    let mut pc: *mut c_char = std::ptr::null_mut();
    let mut lnum: c_int = 0;
    let mut cur_aff: *mut AffheaderT = std::ptr::null_mut();
    let mut did_postpone_prefix = false;
    let mut aff_todo: c_int = 0;
    let mut low: *mut c_char = std::ptr::null_mut();
    let mut fol: *mut c_char = std::ptr::null_mut();
    let mut upp: *mut c_char = std::ptr::null_mut();
    let mut found_map = false;
    let mut compminlen: c_int = 0;
    let mut compsylmax: c_int = 0;
    let mut compoptions: c_int = 0;
    let mut compmax: c_int = 0;
    let mut compflags: *mut c_char = std::ptr::null_mut();
    let mut midword: *mut c_char = std::ptr::null_mut();
    let mut syllable: *mut c_char = std::ptr::null_mut();
    let mut sofofrom: *mut c_char = std::ptr::null_mut();
    let mut sofoto: *mut c_char = std::ptr::null_mut();

    // Read all lines.
    while !vim_fgets(
        rline.as_mut_ptr().cast::<c_char>(),
        MAXLINELEN_AFF as c_int,
        fd,
    ) && !std::ptr::addr_of!(got_int_global).read()
    {
        line_breakcheck_sug();
        lnum += 1;

        // Skip comment lines.
        if rline[0] == b'#' {
            continue;
        }

        // Convert from "SET" encoding to 'encoding' when needed.
        xfree_aff(pc.cast::<c_void>());
        let line: *mut c_char;
        let conv = spin_conv(spin);
        if (*conv).vc_type != CONV_NONE {
            pc = string_convert(
                conv,
                rline.as_mut_ptr().cast::<c_char>(),
                std::ptr::null_mut(),
            );
            if pc.is_null() {
                smsg_aff(
                    0,
                    c"Conversion failure for word in %s line %d: %s".as_ptr(),
                    fname,
                    lnum,
                    rline.as_ptr(),
                );
                continue;
            }
            line = pc;
        } else {
            pc = std::ptr::null_mut();
            line = rline.as_mut_ptr().cast::<c_char>();
        }

        // Split line into whitespace-separated items.
        let mut items: [*mut c_char; MAXITEMCNT] = [std::ptr::null_mut(); MAXITEMCNT];
        let mut itemcnt: usize = 0;
        {
            let mut p = line;
            loop {
                // Skip whitespace and CR/NL.
                while *p != 0 && (*p as u8) <= b' ' {
                    p = p.add(1);
                }
                if *p == 0 {
                    break;
                }
                if itemcnt == MAXITEMCNT {
                    break;
                }
                items[itemcnt] = p;
                itemcnt += 1;
                // Some items have arbitrary text argument; don't split them.
                if itemcnt == 2 && rs_spell_info_item(items[0]) {
                    while (*p as u8) >= b' ' || *p == b'\t' as c_char {
                        p = p.add(1);
                    }
                } else {
                    while (*p as u8) > b' ' {
                        p = p.add(1);
                    }
                }
                if *p == 0 {
                    break;
                }
                *p = 0;
                p = p.add(1);
            }
        }
        let itemcnt = itemcnt as c_int;

        if itemcnt == 0 {
            continue;
        }

        // Convenience: cast items[] to *const *const c_char for rs_is_aff_rule.
        let items_c = items.as_ptr() as *const *const c_char;

        macro_rules! is_rule {
            ($name:expr, $cnt:expr) => {
                rs_is_aff_rule(items_c, itemcnt, $name.as_ptr(), $cnt)
            };
        }

        if is_rule!(c"SET", 2) && (*aff).af_enc.is_null() {
            // Setup encoding conversion.
            (*aff).af_enc = enc_canonize(items[1]);
            let conv = spin_conv(spin);
            if (*spin).si_ascii == 0
                && convert_setup(conv, (*aff).af_enc, std::ptr::addr_of!(p_enc_global).read())
                    == FAIL
            {
                smsg_aff(
                    0,
                    c"Conversion in %s not supported: from %s to %s".as_ptr(),
                    fname,
                    (*aff).af_enc,
                    std::ptr::addr_of!(p_enc_global).read(),
                );
            }
            (*conv).vc_fail = true;
        } else if is_rule!(c"FLAG", 2) && (*aff).af_flagtype == AFT_CHAR {
            if libc::strcmp(items[1], c"long".as_ptr()) == 0 {
                (*aff).af_flagtype = AFT_LONG;
            } else if libc::strcmp(items[1], c"num".as_ptr()) == 0 {
                (*aff).af_flagtype = AFT_NUM;
            } else if libc::strcmp(items[1], c"caplong".as_ptr()) == 0 {
                (*aff).af_flagtype = AFT_CAPLONG;
            } else {
                smsg_aff(
                    0,
                    c"Invalid value for FLAG in %s line %d: %s".as_ptr(),
                    fname,
                    lnum,
                    items[1],
                );
            }
            if (*aff).af_rare != 0
                || (*aff).af_keepcase != 0
                || (*aff).af_bad != 0
                || (*aff).af_needaffix != 0
                || (*aff).af_circumfix != 0
                || (*aff).af_needcomp != 0
                || (*aff).af_comproot != 0
                || (*aff).af_nosuggest != 0
                || !compflags.is_null()
                || (*aff).af_suff.ht_used > 0
                || (*aff).af_pref.ht_used > 0
            {
                smsg_aff(
                    0,
                    c"FLAG after using flags in %s line %d: %s".as_ptr(),
                    fname,
                    lnum,
                    items[1],
                );
            }
        } else if rs_spell_info_item(items[0]) && itemcnt > 1 {
            // info item (NAME, HOME, etc.)
            let si_info_len = if (*spin).si_info.is_null() {
                0
            } else {
                libc::strlen((*spin).si_info.cast::<libc::c_char>())
            };
            let items0_len = libc::strlen(items[0].cast::<libc::c_char>());
            let items1_len = libc::strlen(items[1].cast::<libc::c_char>());
            let total = si_info_len + items0_len + items1_len + 3;
            let p = rs_getroom(spin, total, false).cast::<c_char>();
            if !(*spin).si_info.is_null() {
                libc::strcpy(p, (*spin).si_info);
                libc::strcat(p, c"\n".as_ptr());
            }
            libc::strcat(p, items[0]);
            libc::strcat(p, c" ".as_ptr());
            libc::strcat(p, items[1]);
            (*spin).si_info = p;
        } else if is_rule!(c"MIDWORD", 2) && midword.is_null() {
            midword = rs_getroom_save(spin, items[1]);
        } else if is_rule!(c"TRY", 2) {
            // ignored
        } else if (is_rule!(c"RAR", 2) || is_rule!(c"RARE", 2)) && (*aff).af_rare == 0 {
            (*aff).af_rare = rs_affitem2flag((*aff).af_flagtype, items[1], fname, lnum);
        } else if (is_rule!(c"KEP", 2) || is_rule!(c"KEEPCASE", 2)) && (*aff).af_keepcase == 0 {
            (*aff).af_keepcase = rs_affitem2flag((*aff).af_flagtype, items[1], fname, lnum);
        } else if (is_rule!(c"BAD", 2) || is_rule!(c"FORBIDDENWORD", 2)) && (*aff).af_bad == 0 {
            (*aff).af_bad = rs_affitem2flag((*aff).af_flagtype, items[1], fname, lnum);
        } else if is_rule!(c"NEEDAFFIX", 2) && (*aff).af_needaffix == 0 {
            (*aff).af_needaffix = rs_affitem2flag((*aff).af_flagtype, items[1], fname, lnum);
        } else if is_rule!(c"CIRCUMFIX", 2) && (*aff).af_circumfix == 0 {
            (*aff).af_circumfix = rs_affitem2flag((*aff).af_flagtype, items[1], fname, lnum);
        } else if is_rule!(c"NOSUGGEST", 2) && (*aff).af_nosuggest == 0 {
            (*aff).af_nosuggest = rs_affitem2flag((*aff).af_flagtype, items[1], fname, lnum);
        } else if (is_rule!(c"NEEDCOMPOUND", 2) || is_rule!(c"ONLYINCOMPOUND", 2))
            && (*aff).af_needcomp == 0
        {
            (*aff).af_needcomp = rs_affitem2flag((*aff).af_flagtype, items[1], fname, lnum);
        } else if is_rule!(c"COMPOUNDROOT", 2) && (*aff).af_comproot == 0 {
            (*aff).af_comproot = rs_affitem2flag((*aff).af_flagtype, items[1], fname, lnum);
        } else if is_rule!(c"COMPOUNDFORBIDFLAG", 2) && (*aff).af_compforbid == 0 {
            (*aff).af_compforbid = rs_affitem2flag((*aff).af_flagtype, items[1], fname, lnum);
            if (*aff).af_pref.ht_used > 0 {
                smsg_aff(
                    0,
                    c"Defining COMPOUNDFORBIDFLAG after PFX item may give wrong results in %s line %d".as_ptr(),
                    fname,
                    lnum,
                );
            }
        } else if is_rule!(c"COMPOUNDPERMITFLAG", 2) && (*aff).af_comppermit == 0 {
            (*aff).af_comppermit = rs_affitem2flag((*aff).af_flagtype, items[1], fname, lnum);
            if (*aff).af_pref.ht_used > 0 {
                smsg_aff(
                    0,
                    c"Defining COMPOUNDPERMITFLAG after PFX item may give wrong results in %s line %d".as_ptr(),
                    fname,
                    lnum,
                );
            }
        } else if is_rule!(c"COMPOUNDFLAG", 2) && compflags.is_null() {
            // Turn flag "c" into "c+", "Na" into "Na+", etc.
            let item1_len = libc::strlen(items[1].cast::<libc::c_char>());
            let p = rs_getroom(spin, item1_len + 2, false).cast::<c_char>();
            libc::strcpy(p, items[1]);
            libc::strcat(p, c"+".as_ptr());
            compflags = p;
        } else if is_rule!(c"COMPOUNDRULES", 2) {
            if libc::atoi(items[1]) == 0 {
                smsg_aff(
                    0,
                    c"Wrong COMPOUNDRULES value in %s line %d: %s".as_ptr(),
                    fname,
                    lnum,
                    items[1],
                );
            }
        } else if is_rule!(c"COMPOUNDRULE", 2) {
            // Don't use first rule if it is a number.
            if !compflags.is_null() || *skipdigits(items[1]) != 0 {
                let l = libc::strlen(items[1].cast::<libc::c_char>()) as c_int
                    + 1
                    + if compflags.is_null() {
                        0
                    } else {
                        libc::strlen(compflags.cast::<libc::c_char>()) as c_int + 1
                    };
                let p = rs_getroom(spin, l as usize, false).cast::<c_char>();
                if !compflags.is_null() {
                    libc::strcpy(p, compflags);
                    libc::strcat(p, c"/".as_ptr());
                }
                libc::strcat(p, items[1]);
                compflags = p;
            }
        } else if is_rule!(c"COMPOUNDWORDMAX", 2) && compmax == 0 {
            compmax = libc::atoi(items[1]);
            if compmax == 0 {
                smsg_aff(
                    0,
                    c"Wrong COMPOUNDWORDMAX value in %s line %d: %s".as_ptr(),
                    fname,
                    lnum,
                    items[1],
                );
            }
        } else if is_rule!(c"COMPOUNDMIN", 2) && compminlen == 0 {
            compminlen = libc::atoi(items[1]);
            if compminlen == 0 {
                smsg_aff(
                    0,
                    c"Wrong COMPOUNDMIN value in %s line %d: %s".as_ptr(),
                    fname,
                    lnum,
                    items[1],
                );
            }
        } else if is_rule!(c"COMPOUNDSYLMAX", 2) && compsylmax == 0 {
            compsylmax = libc::atoi(items[1]);
            if compsylmax == 0 {
                smsg_aff(
                    0,
                    c"Wrong COMPOUNDSYLMAX value in %s line %d: %s".as_ptr(),
                    fname,
                    lnum,
                    items[1],
                );
            }
        } else if is_rule!(c"CHECKCOMPOUNDDUP", 1) {
            compoptions |= COMP_CHECKDUP;
        } else if is_rule!(c"CHECKCOMPOUNDREP", 1) {
            compoptions |= COMP_CHECKREP;
        } else if is_rule!(c"CHECKCOMPOUNDCASE", 1) {
            compoptions |= COMP_CHECKCASE;
        } else if is_rule!(c"CHECKCOMPOUNDTRIPLE", 1) {
            compoptions |= COMP_CHECKTRIPLE;
        } else if is_rule!(c"CHECKCOMPOUNDPATTERN", 2) {
            if libc::atoi(items[1]) == 0 {
                smsg_aff(
                    0,
                    c"Wrong CHECKCOMPOUNDPATTERN value in %s line %d: %s".as_ptr(),
                    fname,
                    lnum,
                    items[1],
                );
            }
        } else if is_rule!(c"CHECKCOMPOUNDPATTERN", 3) {
            let gap = &raw mut (*spin).si_comppat;
            // Only add the couple if it isn't already there.
            let mut i: c_int = 0;
            while i < (*gap).ga_len - 1 {
                let data = (*gap).ga_data.cast::<*mut c_char>();
                if libc::strcmp(*data.add(i as usize), items[1]) == 0
                    && libc::strcmp(*data.add(i as usize + 1), items[2]) == 0
                {
                    break;
                }
                i += 2;
            }
            if i >= (*gap).ga_len {
                ga_grow(gap, 2);
                let data = (*gap).ga_data.cast::<*mut c_char>();
                *data.add((*gap).ga_len as usize) = rs_getroom_save(spin, items[1]);
                (*gap).ga_len += 1;
                *data.add((*gap).ga_len as usize) = rs_getroom_save(spin, items[2]);
                (*gap).ga_len += 1;
            }
        } else if is_rule!(c"SYLLABLE", 2) && syllable.is_null() {
            syllable = rs_getroom_save(spin, items[1]);
        } else if is_rule!(c"NOBREAK", 1) {
            (*spin).si_nobreak = 1;
        } else if is_rule!(c"NOSPLITSUGS", 1) {
            (*spin).si_nosplitsugs = 1;
        } else if is_rule!(c"NOCOMPOUNDSUGS", 1) {
            (*spin).si_nocompoundsugs = 1;
        } else if is_rule!(c"NOSUGFILE", 1) {
            (*spin).si_nosugfile = 1;
        } else if is_rule!(c"PFXPOSTPONE", 1) {
            (*aff).af_pfxpostpone = 1;
        } else if is_rule!(c"IGNOREEXTRA", 1) {
            (*aff).af_ignoreextra = true;
        } else if (libc::strcmp(items[0], c"PFX".as_ptr()) == 0
            || libc::strcmp(items[0], c"SFX".as_ptr()) == 0)
            && aff_todo == 0
            && itemcnt >= 4
        {
            // PFX/SFX header: key combine count
            let mut lasti: c_int = 4;
            let tp = if *items[0] == b'P' as c_char {
                &raw mut (*aff).af_pref
            } else {
                &raw mut (*aff).af_suff
            };

            let mut key = [0u8; AH_KEY_LEN];
            xstrlcpy(key.as_mut_ptr().cast::<c_char>(), items[1], AH_KEY_LEN);
            let hi = hash_find(tp, key.as_ptr().cast::<c_char>());
            if !hi.is_null() && !hi_is_empty(hi) {
                cur_aff = (*hi).hi_key.cast::<AffheaderT>();
                let combine_y = *items[2] == b'Y' as c_char;
                if ((*cur_aff).ah_combine != 0) != combine_y {
                    smsg_aff(
                        0,
                        c"Different combining flag in continued affix block in %s line %d: %s"
                            .as_ptr(),
                        fname,
                        lnum,
                        items[1],
                    );
                }
                if (*cur_aff).ah_follows == 0 {
                    smsg_aff(
                        0,
                        c"Duplicate affix in %s line %d: %s".as_ptr(),
                        fname,
                        lnum,
                        items[1],
                    );
                }
            } else {
                // New affix letter.
                cur_aff =
                    rs_getroom(spin, std::mem::size_of::<AffheaderT>(), true).cast::<AffheaderT>();
                (*cur_aff).ah_flag = rs_affitem2flag((*aff).af_flagtype, items[1], fname, lnum);
                if (*cur_aff).ah_flag == 0
                    || libc::strlen(items[1].cast::<libc::c_char>()) >= AH_KEY_LEN
                {
                    break;
                }
                if (*cur_aff).ah_flag == (*aff).af_bad
                    || (*cur_aff).ah_flag == (*aff).af_rare
                    || (*cur_aff).ah_flag == (*aff).af_keepcase
                    || (*cur_aff).ah_flag == (*aff).af_needaffix
                    || (*cur_aff).ah_flag == (*aff).af_circumfix
                    || (*cur_aff).ah_flag == (*aff).af_nosuggest
                    || (*cur_aff).ah_flag == (*aff).af_needcomp
                    || (*cur_aff).ah_flag == (*aff).af_comproot
                {
                    smsg_aff(
                        0,
                        c"Affix also used for BAD/RARE/KEEPCASE/NEEDAFFIX/NEEDCOMPOUND/NOSUGGEST in %s line %d: %s".as_ptr(),
                        fname,
                        lnum,
                        items[1],
                    );
                }
                xstrlcpy((*cur_aff).ah_key.as_mut_ptr(), items[1], AH_KEY_LEN);
                hash_add(tp, (*cur_aff).ah_key.as_mut_ptr());
                (*cur_aff).ah_combine = c_int::from(*items[2] == b'Y' as c_char);
            }

            // Check for the "S" flag (more blocks following).
            if itemcnt > lasti && libc::strcmp(items[lasti as usize], c"S".as_ptr()) == 0 {
                lasti += 1;
                (*cur_aff).ah_follows = 1;
            } else {
                (*cur_aff).ah_follows = 0;
            }

            // Warn about trailing items (unless IGNOREEXTRA or starts with #).
            if itemcnt > lasti && !(*aff).af_ignoreextra && *items[lasti as usize] != b'#' as c_char
            {
                smsg_aff(
                    0,
                    c"Trailing text in %s line %d: %s".as_ptr(),
                    fname,
                    lnum,
                    items[lasti as usize],
                );
            }

            if libc::strcmp(items[2], c"Y".as_ptr()) != 0
                && libc::strcmp(items[2], c"N".as_ptr()) != 0
            {
                smsg_aff(
                    0,
                    c"Expected Y or N in %s line %d: %s".as_ptr(),
                    fname,
                    lnum,
                    items[2],
                );
            }

            if *items[0] == b'P' as c_char && (*aff).af_pfxpostpone != 0 {
                if (*cur_aff).ah_newID == 0 {
                    rs_check_renumber(spin);
                    (*spin).si_newprefID += 1;
                    (*cur_aff).ah_newID = (*spin).si_newprefID;
                    did_postpone_prefix = false;
                } else {
                    did_postpone_prefix = true;
                }
            }

            aff_todo = libc::atoi(items[3]);
        } else if (libc::strcmp(items[0], c"PFX".as_ptr()) == 0
            || libc::strcmp(items[0], c"SFX".as_ptr()) == 0)
            && aff_todo > 0
            && !cur_aff.is_null()
            && libc::strcmp((*cur_aff).ah_key.as_ptr(), items[1]) == 0
            && itemcnt >= 5
        {
            // PFX/SFX entry: key chop add condition [#comment]
            let lasti: c_int = 5;

            // Warn about trailing items.
            if itemcnt > lasti
                && *items[lasti as usize] != b'#' as c_char
                && (libc::strcmp(items[lasti as usize], c"-".as_ptr()) != 0 || itemcnt != lasti + 1)
            {
                smsg_aff(
                    0,
                    c"Trailing text in %s line %d: %s".as_ptr(),
                    fname,
                    lnum,
                    items[lasti as usize],
                );
            }

            aff_todo -= 1;
            let aff_entry =
                rs_getroom(spin, std::mem::size_of::<AffentryT>(), true).cast::<AffentryT>();

            if libc::strcmp(items[2], c"0".as_ptr()) != 0 {
                (*aff_entry).ae_chop = rs_getroom_save(spin, items[2]);
            }
            if libc::strcmp(items[3], c"0".as_ptr()) != 0 {
                (*aff_entry).ae_add = rs_getroom_save(spin, items[3]);

                // Recognize flags on the affix: abcd/XYZ
                (*aff_entry).ae_flags = vim_strchr_aff((*aff_entry).ae_add, c_int::from(b'/'));
                if !(*aff_entry).ae_flags.is_null() {
                    *(*aff_entry).ae_flags = 0;
                    (*aff_entry).ae_flags = (*aff_entry).ae_flags.add(1);
                    rs_aff_process_flags(aff, aff_entry);
                }
            }

            // Don't use affix entry with non-ASCII chars when si_ascii is set.
            if (*spin).si_ascii == 0
                || (!has_non_ascii((*aff_entry).ae_chop) && !has_non_ascii((*aff_entry).ae_add))
            {
                (*aff_entry).ae_next = (*cur_aff).ah_first;
                (*cur_aff).ah_first = aff_entry;

                if libc::strcmp(items[4], c".".as_ptr()) != 0 {
                    let mut buf = [0u8; MAXLINELEN_AFF];
                    (*aff_entry).ae_cond = rs_getroom_save(spin, items[4]);
                    let fmt = if *items[0] == b'P' as c_char {
                        c"^%s".as_ptr()
                    } else {
                        c"%s$".as_ptr()
                    };
                    vim_snprintf(
                        buf.as_mut_ptr().cast::<c_char>(),
                        MAXLINELEN_AFF,
                        fmt,
                        items[4],
                    );
                    (*aff_entry).ae_prog = vim_regcomp(
                        buf.as_ptr().cast::<c_char>(),
                        RE_MAGIC + RE_STRING + RE_STRICT,
                    );
                    if (*aff_entry).ae_prog.is_null() {
                        smsg_aff(
                            0,
                            c"Broken condition in %s line %d: %s".as_ptr(),
                            fname,
                            lnum,
                            items[4],
                        );
                    }
                }

                // For postponed prefixes add an entry in si_prefcond.
                if *items[0] == b'P' as c_char
                    && (*aff).af_pfxpostpone != 0
                    && (*aff_entry).ae_flags.is_null()
                {
                    let mut upper = false;

                    // Check if chop is one lower-case letter and add ends in upper-case.
                    if !(*aff_entry).ae_chop.is_null()
                        && !(*aff_entry).ae_add.is_null()
                        && *(*aff_entry)
                            .ae_chop
                            .add(utfc_ptr2len_spell((*aff_entry).ae_chop) as usize)
                            == 0
                    {
                        let c = utf_ptr2char((*aff_entry).ae_chop);
                        let c_up = spell_toupper(c);
                        if c_up != c
                            && ((*aff_entry).ae_cond.is_null()
                                || utf_ptr2char((*aff_entry).ae_cond) == c)
                        {
                            let add_len = libc::strlen((*aff_entry).ae_add.cast::<libc::c_char>());
                            let mut p = (*aff_entry).ae_add.add(add_len);
                            // MB_PTR_BACK: back up one multibyte char
                            let off = utf_head_off((*aff_entry).ae_add, p.sub(1)) as usize;
                            p = p.sub(off + 1);
                            if utf_ptr2char(p) == c_up {
                                upper = true;
                                (*aff_entry).ae_chop = std::ptr::null_mut();
                                *p = 0;

                                if !(*aff_entry).ae_cond.is_null() {
                                    let mut cbuf = [0u8; MAXLINELEN_AFF];
                                    onecap_copy(items[4], cbuf.as_mut_ptr().cast::<c_char>(), true);
                                    (*aff_entry).ae_cond =
                                        rs_getroom_save(spin, cbuf.as_mut_ptr().cast::<c_char>());
                                    if !(*aff_entry).ae_cond.is_null() {
                                        vim_snprintf(
                                            cbuf.as_mut_ptr().cast::<c_char>(),
                                            MAXLINELEN_AFF,
                                            c"^%s".as_ptr(),
                                            (*aff_entry).ae_cond,
                                        );
                                        vim_regfree_spell((*aff_entry).ae_prog);
                                        (*aff_entry).ae_prog = vim_regcomp(
                                            cbuf.as_ptr().cast::<c_char>(),
                                            RE_MAGIC + RE_STRING,
                                        );
                                    }
                                }
                            }
                        }
                    }

                    if (*aff_entry).ae_chop.is_null() {
                        // Find a previously used condition.
                        let mut idx: c_int = (*spin).si_prefcond.ga_len - 1;
                        while idx >= 0 {
                            let pp = *((*spin).si_prefcond.ga_data.cast::<*mut c_char>())
                                .add(idx as usize);
                            if rs_str_equal(pp, (*aff_entry).ae_cond) {
                                break;
                            }
                            idx -= 1;
                        }
                        if idx < 0 {
                            // Not found; add a new condition.
                            idx = (*spin).si_prefcond.ga_len;
                            ga_grow(&raw mut (*spin).si_prefcond, 1);
                            let pp = (*spin)
                                .si_prefcond
                                .ga_data
                                .cast::<*mut c_char>()
                                .add((*spin).si_prefcond.ga_len as usize);
                            *pp = if (*aff_entry).ae_cond.is_null() {
                                std::ptr::null_mut()
                            } else {
                                rs_getroom_save(spin, (*aff_entry).ae_cond)
                            };
                            (*spin).si_prefcond.ga_len += 1;
                        }

                        // Add prefix to prefix tree.
                        let p_add = if (*aff_entry).ae_add.is_null() {
                            c"".as_ptr()
                        } else {
                            (*aff_entry).ae_add.cast_const()
                        };

                        let mut n = PFX_FLAGS;
                        if (*cur_aff).ah_combine == 0 {
                            n |= WFP_NC;
                        }
                        if upper {
                            n |= WFP_UP;
                        }
                        if (*aff_entry).ae_comppermit != 0 {
                            n |= WFP_COMPPERMIT;
                        }
                        if (*aff_entry).ae_compforbid != 0 {
                            n |= WFP_COMPFORBID;
                        }
                        rs_tree_add_word(
                            spin,
                            p_add,
                            (*spin).si_prefroot,
                            n,
                            idx,
                            (*cur_aff).ah_newID,
                        );
                        did_postpone_prefix = true;
                    }

                    // Didn't use ah_newID: undo.
                    if aff_todo == 0 && !did_postpone_prefix {
                        (*spin).si_newprefID -= 1;
                        (*cur_aff).ah_newID = 0;
                    }
                }
            }
        } else if is_rule!(c"FOL", 2) && fol.is_null() {
            fol = xstrdup(items[1]);
        } else if is_rule!(c"LOW", 2) && low.is_null() {
            low = xstrdup(items[1]);
        } else if is_rule!(c"UPP", 2) && upp.is_null() {
            upp = xstrdup(items[1]);
        } else if is_rule!(c"REP", 2) || is_rule!(c"REPSAL", 2) {
            // Count line: ignored if numeric, warn otherwise.
            if !(*items[1] as u8).is_ascii_digit() {
                smsg_aff(
                    0,
                    c"Expected REP(SAL) count in %s line %d".as_ptr(),
                    fname,
                    lnum,
                );
            }
        } else if (libc::strcmp(items[0], c"REP".as_ptr()) == 0
            || libc::strcmp(items[0], c"REPSAL".as_ptr()) == 0)
            && itemcnt >= 3
        {
            // REP/REPSAL item.
            if itemcnt > 3 && *items[3] != b'#' as c_char {
                smsg_aff(
                    0,
                    c"Trailing text in %s line %d: %s".as_ptr(),
                    fname,
                    lnum,
                    items[3],
                );
            }
            let do_this = if *items[0].add(3) == b'S' as c_char {
                do_repsal
            } else {
                do_rep
            };
            if do_this {
                // Replace '_' with space.
                let mut p = items[1];
                while *p != 0 {
                    if *p == b'_' as c_char {
                        *p = b' ' as c_char;
                    }
                    let step = utfc_ptr2len_spell(p) as usize;
                    p = p.add(step);
                }
                let mut p = items[2];
                while *p != 0 {
                    if *p == b'_' as c_char {
                        *p = b' ' as c_char;
                    }
                    let step = utfc_ptr2len_spell(p) as usize;
                    p = p.add(step);
                }
                let gap = if *items[0].add(3) == b'S' as c_char {
                    &raw mut (*spin).si_repsal
                } else {
                    &raw mut (*spin).si_rep
                };
                rs_add_fromto(spin, gap, items[1], items[2]);
            }
        } else if is_rule!(c"MAP", 2) {
            if !found_map {
                found_map = true;
                if !(*items[1] as u8).is_ascii_digit() {
                    smsg_aff(0, c"Expected MAP count in %s line %d".as_ptr(), fname, lnum);
                }
            } else if do_mapline {
                // Check that every character appears only once.
                let mut p: *const c_char = items[1];
                while *p != 0 {
                    let c = mb_ptr2char_adv_p(&raw mut p);
                    let si_map_data = (*spin).si_map.ga_data.cast::<c_char>();
                    let si_map_len = (*spin).si_map.ga_len;
                    // Check against existing map data (not empty).
                    let in_map = if si_map_len > 0 {
                        !vim_strchr_aff(si_map_data, c).is_null()
                    } else {
                        false
                    };
                    let in_rest = !vim_strchr_aff(p, c).is_null();
                    if in_map || in_rest {
                        smsg_aff(
                            0,
                            c"Duplicate character in MAP in %s line %d".as_ptr(),
                            fname,
                            lnum,
                        );
                    }
                }
                ga_concat(&raw mut (*spin).si_map, items[1]);
                ga_append(&raw mut (*spin).si_map, b'/' as c_char);
            }
        } else if is_rule!(c"SAL", 3) {
            if do_sal {
                if libc::strcmp(items[1], c"followup".as_ptr()) == 0 {
                    (*spin).si_followup = c_int::from(crate::rs_sal_to_bool(items[2]));
                } else if libc::strcmp(items[1], c"collapse_result".as_ptr()) == 0 {
                    (*spin).si_collapse = c_int::from(crate::rs_sal_to_bool(items[2]));
                } else if libc::strcmp(items[1], c"remove_accents".as_ptr()) == 0 {
                    (*spin).si_rem_accents = c_int::from(crate::rs_sal_to_bool(items[2]));
                } else {
                    // from-to pair; "_" means empty.
                    let to = if libc::strcmp(items[2], c"_".as_ptr()) == 0 {
                        c"".as_ptr()
                    } else {
                        items[2].cast_const()
                    };
                    rs_add_fromto(spin, &raw mut (*spin).si_sal, items[1], to);
                }
            }
        } else if is_rule!(c"SOFOFROM", 2) && sofofrom.is_null() {
            sofofrom = rs_getroom_save(spin, items[1]);
        } else if is_rule!(c"SOFOTO", 2) && sofoto.is_null() {
            sofoto = rs_getroom_save(spin, items[1]);
        } else if libc::strcmp(items[0], c"COMMON".as_ptr()) == 0 {
            for item in items.iter().take(itemcnt as usize).skip(1) {
                let hi = hash_find(&raw const (*spin).si_commonwords, (*item).cast_const());
                if hi_is_empty(hi) {
                    let p = xstrdup(*item).cast::<c_char>();
                    hash_add(&raw mut (*spin).si_commonwords, p);
                }
            }
        } else {
            smsg_aff(
                0,
                c"Unrecognized or duplicate item in %s line %d: %s".as_ptr(),
                fname,
                lnum,
                items[0],
            );
        }
    } // end while (read lines)

    xfree_aff(pc.cast::<c_void>());

    // Post-loop: handle FOL/LOW/UPP.
    if !fol.is_null() || !low.is_null() || !upp.is_null() {
        if (*spin).si_clear_chartab != 0 {
            init_spell_chartab();
            (*spin).si_clear_chartab = 0;
        }
        xfree_aff(fol.cast::<c_void>());
        xfree_aff(low.cast::<c_void>());
        xfree_aff(upp.cast::<c_void>());
    }

    // Use compound specifications from .aff for spell info.
    if compmax != 0 {
        rs_aff_check_number((*spin).si_compmax, compmax, c"COMPOUNDWORDMAX".as_ptr());
        (*spin).si_compmax = compmax;
    }
    if compminlen != 0 {
        rs_aff_check_number((*spin).si_compminlen, compminlen, c"COMPOUNDMIN".as_ptr());
        (*spin).si_compminlen = compminlen;
    }
    if compsylmax != 0 {
        if syllable.is_null() {
            smsg_aff(
                0,
                c"%s".as_ptr(),
                c"COMPOUNDSYLMAX used without SYLLABLE".as_ptr(),
            );
        }
        rs_aff_check_number(
            (*spin).si_compsylmax,
            compsylmax,
            c"COMPOUNDSYLMAX".as_ptr(),
        );
        (*spin).si_compsylmax = compsylmax;
    }
    if compoptions != 0 {
        rs_aff_check_number(
            (*spin).si_compoptions,
            compoptions,
            c"COMPOUND options".as_ptr(),
        );
        (*spin).si_compoptions |= compoptions;
    }
    if !compflags.is_null() {
        rs_process_compflags(spin, aff, compflags);
    }

    // Check that we didn't use too many renumbered flags.
    if (*spin).si_newcompID < (*spin).si_newprefID {
        if (*spin).si_newcompID == 127 || (*spin).si_newcompID == 255 {
            msg(c"Too many postponed prefixes".as_ptr(), 0);
        } else if (*spin).si_newprefID == 0 || (*spin).si_newprefID == 127 {
            msg(c"Too many compound flags".as_ptr(), 0);
        } else {
            msg(
                c"Too many postponed prefixes and/or compound flags".as_ptr(),
                0,
            );
        }
    }

    if !syllable.is_null() {
        rs_aff_check_string((*spin).si_syllable, syllable, c"SYLLABLE".as_ptr());
        (*spin).si_syllable = syllable;
    }

    if !sofofrom.is_null() || !sofoto.is_null() {
        if sofofrom.is_null() || sofoto.is_null() {
            smsg_aff(
                0,
                c"Missing SOFO%s line in %s".as_ptr(),
                if sofofrom.is_null() {
                    c"FROM".as_ptr()
                } else {
                    c"TO".as_ptr()
                },
                fname,
            );
        } else if (*spin).si_sal.ga_len > 0 {
            smsg_aff(0, c"Both SAL and SOFO lines in %s".as_ptr(), fname);
        } else {
            rs_aff_check_string((*spin).si_sofofr, sofofrom, c"SOFOFROM".as_ptr());
            rs_aff_check_string((*spin).si_sofoto, sofoto, c"SOFOTO".as_ptr());
            (*spin).si_sofofr = sofofrom;
            (*spin).si_sofoto = sofoto;
        }
    }

    if !midword.is_null() {
        rs_aff_check_string((*spin).si_midword, midword, c"MIDWORD".as_ptr());
        (*spin).si_midword = midword;
    }

    libc::fclose(fd);
    aff
}

// =============================================================================
// Phase 7: write_vim_spell and mkspell -- top-level spell file creation
// =============================================================================

extern "C" {
    #[link_name = "os_fopen"]
    fn os_fopen_wvs(path: *const c_char, mode: *const c_char) -> *mut libc::FILE;
    #[link_name = "semsg"]
    fn semsg_wvs(fmt: *const c_char, ...) -> bool;
    #[link_name = "put_bytes"]
    fn put_bytes_wvs(fd: *mut libc::FILE, number: u64, len: usize) -> bool;
    #[link_name = "emsg"]
    fn emsg_wvs(s: *const c_char) -> bool;
    // C-only helpers
    fn os_path_exists(name: *const c_char) -> bool;
    fn os_isdir(name: *const c_char) -> bool;
    #[link_name = "hash_clear_all"]
    fn hash_clear_all_wvs(ht: *mut crate::HashtabRaw, off: u32);
    #[link_name = "ga_init"]
    fn ga_init_wvs(gap: *mut crate::GArrayRaw, itemsize: c_int, growsize: c_int);
    #[link_name = "ga_clear"]
    fn ga_clear_wvs(gap: *mut crate::GArrayRaw);
    #[link_name = "hash_init"]
    fn hash_init_wvs(ht: *mut crate::HashtabRaw);
    #[link_name = "convert_setup"]
    fn convert_setup_wvs(vcp: *mut VimconvT, from: *mut c_char, to: *mut c_char) -> c_int;
    #[link_name = "vim_snprintf"]
    fn vim_snprintf_wvs(str_: *mut c_char, str_m: usize, fmt: *const c_char, ...) -> c_int;
    #[link_name = "IObuff"]
    static mut IObuff_wvs: [c_char; 1025];
    #[link_name = "path_tail"]
    fn path_tail_wvs(fname: *const c_char) -> *mut c_char;
    #[link_name = "vim_strchr"]
    fn vim_strchr_wvs(s: *const c_char, c: c_int) -> *mut c_char;
    #[link_name = "xmalloc"]
    fn xmalloc_wvs(size: usize) -> *mut c_void;
    #[link_name = "xfree"]
    fn xfree_wvs(ptr: *mut c_void);
    #[link_name = "xstrlcpy"]
    fn xstrlcpy_wvs(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize;
    #[link_name = "msg"]
    fn msg_wvs(s: *const c_char, hl_id: c_int) -> bool;
    #[link_name = "got_int"]
    static got_int_wvs: bool;
    fn spell_enc() -> *const c_char;
}

static E_NOTOPEN_WVS: &[u8] = b"E484: Can't open file %s\0";
static E_WRITE_WVS: &[u8] = b"E514: Write failed (file system full?)\0";
static E_INVARG_WVS: &[u8] = b"E474: Invalid argument\0";
static E_EXISTS_WVS: &[u8] = b"E739: Cannot create directory\0";
static E_ISADIR2_WVS: &[u8] = b"E17: \"%s\" is a directory\0";
static E_REGION_WVS: &[u8] = b"E751: Output file name must not have region name\0";
static E_TOOMANY_WVS: &[u8] = b"E754: Only up to %d regions supported\0";
static E_INVREGION_WVS: &[u8] = b"E755: Invalid region in %s\0";
static MSG_COMPOUND_NOBREAK_WVS: &[u8] = b"Warning: both compounding and NOBREAK specified\0";
static MSG_COMPRESS_WVS: &[u8] = b"Compressing word tree...\0";
static MSG_WRITING_WVS: &[u8] = b"Writing spell file %s...\0";
static MSG_DONE_WVS: &[u8] = b"Done!\0";
static MSG_MEMUSE_WVS: &[u8] = b"Estimated runtime memory use: %d bytes\0";

/// CF_WORD / CF_UPPER flags for SN_CHARFLAGS section.
const CF_WORD: u8 = 0x01;
const CF_UPPER: u8 = 0x02;

/// Write the Vim .spl file to `fname`.
///
/// This is the Rust implementation of the C `write_vim_spell` function.
///
/// # Safety
/// `spin` and `fname` must be valid non-null pointers.
/// `fname` must be a NUL-terminated path.
#[no_mangle]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_write_vim_spell(spin: *mut SpellinfoT, fname: *mut c_char) -> c_int {
    let mut retval: c_int = OK;

    let fd = os_fopen_wvs(fname, c"w".as_ptr());
    if fd.is_null() {
        semsg_wvs(E_NOTOPEN_WVS.as_ptr().cast::<c_char>(), fname);
        return FAIL;
    }

    // <HEADER>: <fileID> <versionnr>
    let mut fwv: usize = libc::fwrite(
        VIMSPELLMAGIC.as_ptr().cast::<c_void>(),
        VIMSPELLMAGIC.len(),
        1,
        fd,
    );
    if fwv != 1 {
        retval = FAIL;
        // goto theend
    }
    libc::fputc(c_int::from(VIMSPELLVERSION), fd);

    // Compute regionmask.
    let regionmask: c_int = if (*spin).si_region_count > 1 {
        (1 << (*spin).si_region_count) - 1
    } else {
        0
    };

    // SN_CHARFLAGS section (only when not ascii and not add file).
    if retval == OK && (*spin).si_ascii == 0 && (*spin).si_add == 0 {
        // Build fold chars buffer (max 128*8 bytes).
        let mut folchars = [0u8; 128 * 8];
        let mut l: usize = 0;
        for i in 128usize..256 {
            let n = utf_char2bytes(
                c_int::from(spelltab_global_sf.st_fold[i]),
                folchars.as_mut_ptr().add(l).cast::<c_char>(),
            );
            l += n as usize;
        }

        libc::fputc(c_int::from(SN_CHARFLAGS), fd);
        libc::fputc(c_int::from(SNF_REQUIRED), fd);
        // Section length: 1 (cnt) + 128 (flags) + 2 (follen) + l (fold bytes)
        put_bytes_wvs(fd, (1 + 128 + 2 + l) as u64, 4);

        libc::fputc(128, fd);
        for i in 128usize..256 {
            let mut flags: u8 = 0;
            if spelltab_global_sf.st_isw[i] {
                flags |= CF_WORD;
            }
            if spelltab_global_sf.st_isu[i] {
                flags |= CF_UPPER;
            }
            libc::fputc(c_int::from(flags), fd);
        }
        put_bytes_wvs(fd, l as u64, 2);
        fwv &= libc::fwrite(folchars.as_ptr().cast::<c_void>(), l, 1, fd);
    }

    // All other sections: call rs_write_spell_sections.
    if retval == OK {
        // Sort REP and REPSAL.
        if (*spin).si_rep.ga_len > 0 {
            libc::qsort(
                (*spin).si_rep.ga_data,
                (*spin).si_rep.ga_len as libc::size_t,
                std::mem::size_of::<FromtoC>(),
                Some(rs_rep_compare),
            );
        }
        if (*spin).si_repsal.ga_len > 0 {
            libc::qsort(
                (*spin).si_repsal.ga_data,
                (*spin).si_repsal.ga_len as libc::size_t,
                std::mem::size_of::<FromtoC>(),
                Some(rs_rep_compare),
            );
        }

        // Set si_sugtime for SN_SUGFILE section.
        let sugtime: i64 = if (*spin).si_nosugfile == 0
            && (!(*spin).si_sal.ga_data.is_null() && (*spin).si_sal.ga_len > 0
                || (!(*spin).si_sofofr.is_null() && !(*spin).si_sofoto.is_null()))
        {
            (*spin).si_sugtime = libc::time(std::ptr::null_mut());
            (*spin).si_sugtime
        } else {
            0
        };

        // Build flat pointer arrays for REP.
        let rep_count = (*spin).si_rep.ga_len as usize;
        let rep_from: *mut *const u8 = if rep_count > 0 {
            xmalloc_wvs(rep_count * std::mem::size_of::<*const u8>()).cast()
        } else {
            std::ptr::null_mut()
        };
        let rep_to: *mut *const u8 = if rep_count > 0 {
            xmalloc_wvs(rep_count * std::mem::size_of::<*const u8>()).cast()
        } else {
            std::ptr::null_mut()
        };
        for i in 0..rep_count {
            let ftp = ((*spin).si_rep.ga_data as *mut FromtoC).add(i);
            *rep_from.add(i) = (*ftp).ft_from.cast::<u8>();
            *rep_to.add(i) = (*ftp).ft_to.cast::<u8>();
        }

        // SAL or SOFO selection.
        let use_sal = (*spin).si_sofofr.is_null() || (*spin).si_sofoto.is_null();
        let sal_count = if use_sal {
            (*spin).si_sal.ga_len as usize
        } else {
            0
        };
        let sal_from: *mut *const u8 = if sal_count > 0 {
            xmalloc_wvs(sal_count * std::mem::size_of::<*const u8>()).cast()
        } else {
            std::ptr::null_mut()
        };
        let sal_to: *mut *const u8 = if sal_count > 0 {
            xmalloc_wvs(sal_count * std::mem::size_of::<*const u8>()).cast()
        } else {
            std::ptr::null_mut()
        };
        for i in 0..sal_count {
            let ftp = ((*spin).si_sal.ga_data as *mut FromtoC).add(i);
            *sal_from.add(i) = (*ftp).ft_from.cast::<u8>();
            *sal_to.add(i) = (*ftp).ft_to.cast::<u8>();
        }
        let mut sal_flags: u8 = 0;
        if (*spin).si_followup != 0 {
            sal_flags |= 1;
        } // SAL_F0LLOWUP
        if (*spin).si_collapse != 0 {
            sal_flags |= 2;
        } // SAL_COLLAPSE
        if (*spin).si_rem_accents != 0 {
            sal_flags |= 4;
        } // SAL_REM_ACCENTS

        // Build flat pointer arrays for REPSAL.
        let repsal_count = (*spin).si_repsal.ga_len as usize;
        let repsal_from: *mut *const u8 = if repsal_count > 0 {
            xmalloc_wvs(repsal_count * std::mem::size_of::<*const u8>()).cast()
        } else {
            std::ptr::null_mut()
        };
        let repsal_to: *mut *const u8 = if repsal_count > 0 {
            xmalloc_wvs(repsal_count * std::mem::size_of::<*const u8>()).cast()
        } else {
            std::ptr::null_mut()
        };
        for i in 0..repsal_count {
            let ftp = ((*spin).si_repsal.ga_data as *mut FromtoC).add(i);
            *repsal_from.add(i) = (*ftp).ft_from.cast::<u8>();
            *repsal_to.add(i) = (*ftp).ft_to.cast::<u8>();
        }

        // Build flat pointer arrays for prefcond.
        let prefcond_count = (*spin).si_prefcond.ga_len as usize;
        let prefcond_strs: *mut *const u8 = if prefcond_count > 0 {
            xmalloc_wvs(prefcond_count * std::mem::size_of::<*const u8>()).cast()
        } else {
            std::ptr::null_mut()
        };
        for i in 0..prefcond_count {
            let p = *((*spin).si_prefcond.ga_data as *mut *mut c_char).add(i);
            *prefcond_strs.add(i) = if p.is_null() {
                c"".as_ptr().cast::<u8>()
            } else {
                p.cast::<u8>()
            };
        }

        // Build flat pointer arrays for comppat.
        let comppat_count = (*spin).si_comppat.ga_len as usize;
        let comppat_strs: *mut *const u8 = if comppat_count > 0 {
            xmalloc_wvs(comppat_count * std::mem::size_of::<*const u8>()).cast()
        } else {
            std::ptr::null_mut()
        };
        for i in 0..comppat_count {
            let p = *((*spin).si_comppat.ga_data as *mut *mut c_char).add(i);
            *comppat_strs.add(i) = if p.is_null() {
                c"".as_ptr().cast::<u8>()
            } else {
                p.cast::<u8>()
            };
        }

        let params = SpellSectionParams {
            si_info: (*spin).si_info,
            si_region_count: (*spin).si_region_count,
            si_region_name: (*spin).si_region_name.as_ptr().cast::<u8>(),
            si_skip_charflags: false,
            si_midword: (*spin).si_midword,
            si_prefcond_strs: prefcond_strs.cast_const(),
            si_prefcond_count: (*spin).si_prefcond.ga_len,
            si_rep_from: rep_from.cast_const(),
            si_rep_to: rep_to.cast_const(),
            si_rep_count: (*spin).si_rep.ga_len,
            si_use_sal: use_sal,
            si_sal_from: sal_from.cast_const(),
            si_sal_to: sal_to.cast_const(),
            si_sal_count: (*spin).si_sal.ga_len,
            si_sal_flags: sal_flags,
            si_repsal_from: repsal_from.cast_const(),
            si_repsal_to: repsal_to.cast_const(),
            si_repsal_count: (*spin).si_repsal.ga_len,
            si_sofofr: (*spin).si_sofofr,
            si_sofoto: (*spin).si_sofoto,
            si_map_data: (*spin).si_map.ga_data.cast::<u8>(),
            si_map_len: (*spin).si_map.ga_len,
            si_sugtime: sugtime,
            si_nosplitsugs: (*spin).si_nosplitsugs != 0,
            si_nocompoundsugs: (*spin).si_nocompoundsugs != 0,
            si_compflags: (*spin).si_compflags,
            si_compmax: (*spin).si_compmax as u8,
            si_compminlen: (*spin).si_compminlen as u8,
            si_compsylmax: (*spin).si_compsylmax as u8,
            si_compoptions: (*spin).si_compoptions as u16,
            si_comppat_strs: comppat_strs.cast_const(),
            si_comppat_count: (*spin).si_comppat.ga_len,
            si_nobreak: (*spin).si_nobreak != 0,
            si_syllable: (*spin).si_syllable,
        };

        // Allocate output buffer (256 KB is plenty for all sections).
        let sec_buf_len: usize = 256 * 1024;
        let sec_buf = xmalloc_wvs(sec_buf_len).cast::<u8>();
        let mut written: usize = 0;
        let rs_ret =
            rs_write_spell_sections(&raw const params, sec_buf, sec_buf_len, &raw mut written);
        if rs_ret != 0
            || (written > 0 && libc::fwrite(sec_buf.cast::<c_void>(), written, 1, fd) != 1)
        {
            retval = FAIL;
        }

        xfree_wvs(sec_buf.cast::<c_void>());
        xfree_wvs(rep_from.cast::<c_void>());
        xfree_wvs(rep_to.cast::<c_void>());
        xfree_wvs(sal_from.cast::<c_void>());
        xfree_wvs(sal_to.cast::<c_void>());
        xfree_wvs(repsal_from.cast::<c_void>());
        xfree_wvs(repsal_to.cast::<c_void>());
        xfree_wvs(prefcond_strs.cast::<c_void>());
        xfree_wvs(comppat_strs.cast::<c_void>());
    }

    // SN_WORDS section: iterate si_commonwords hashtable.
    if retval == OK && (*spin).si_commonwords.ht_used > 0 {
        libc::fputc(c_int::from(SN_WORDS), fd);
        libc::fputc(0, fd);

        // Two passes: round 1 = count bytes, round 2 = write bytes.
        for round in 1u32..=2 {
            let mut todo = (*spin).si_commonwords.ht_used;
            let mut len: usize = 0;
            let mut hi = (*spin).si_commonwords.ht_array;
            while todo > 0 {
                // Check if item is empty (null key or removed sentinel).
                let is_empty = (*hi).hi_key.is_null()
                    || std::ptr::eq(
                        (*hi).hi_key,
                        std::ptr::addr_of!(hash_removed_sentinel).cast_mut(),
                    );
                if !is_empty {
                    let word_len = libc::strlen((*hi).hi_key) + 1;
                    len += word_len;
                    if round == 2 {
                        fwv &= libc::fwrite((*hi).hi_key.cast::<c_void>(), word_len, 1, fd);
                    }
                    todo -= 1;
                }
                hi = hi.add(1);
            }
            if round == 1 {
                put_bytes_wvs(fd, len as u64, 4);
            }
        }
    }

    // End of <SECTIONS>.
    if retval == OK {
        libc::fputc(c_int::from(SN_END), fd);
    }

    // <LWORDTREE> <KWORDTREE> <PREFIXTREE>
    if retval == OK {
        (*spin).si_memtot = 0;
        for round in 1u32..=3 {
            let tree: *mut WordnodeT = match round {
                1 => (*(*spin).si_foldroot).wn_sibling,
                2 => (*(*spin).si_keeproot).wn_sibling,
                _ => (*(*spin).si_prefroot).wn_sibling,
            };
            let prefixtree = round == 3;

            // Count pass.
            clear_node_inner(tree);
            let nodecount = put_node_inner(tree, &mut Vec::new(), 0, regionmask, prefixtree);
            if nodecount < 0 {
                retval = FAIL;
                break;
            }

            put_bytes_wvs(fd, nodecount as u64, 4);
            (*spin).si_memtot += nodecount + (nodecount * std::mem::size_of::<c_int>() as c_int);

            // Write pass.
            let tree_buf_len = nodecount as usize * 8 + 1024;
            let tree_buf = xmalloc_wvs(tree_buf_len).cast::<u8>();
            clear_node_inner(tree);
            let mut out_vec: Vec<u8> = Vec::with_capacity(tree_buf_len);
            let nodecount2 = put_node_inner(tree, &mut out_vec, 0, regionmask, prefixtree);
            if nodecount2 < 0 {
                xfree_wvs(tree_buf.cast::<c_void>());
                retval = FAIL;
                break;
            }
            if !out_vec.is_empty()
                && libc::fwrite(out_vec.as_ptr().cast::<c_void>(), out_vec.len(), 1, fd) != 1
            {
                xfree_wvs(tree_buf.cast::<c_void>());
                retval = FAIL;
                break;
            }
            xfree_wvs(tree_buf.cast::<c_void>());
        }
    }

    // Final error-check byte.
    if retval == OK && libc::fputc(0, fd) == libc::EOF {
        retval = FAIL;
    }

    if libc::fclose(fd) == libc::EOF {
        retval = FAIL;
    }
    if fwv != 1 {
        retval = FAIL;
    }
    if retval == FAIL {
        emsg_wvs(E_WRITE_WVS.as_ptr().cast::<c_char>());
    }

    retval
}

/// Create a Vim spell file from one or more word lists.
///
/// This is the Rust implementation of the C `mkspell` function.
///
/// - `fnames[0]` is the output file name.
/// - `fnames[fcount - 1]` is the last input file name.
/// - Exception: when `fnames[0]` ends in `.add`, it's used as the input
///   file name and `.spl` is appended to make the output file name.
///
/// # Safety
/// `fnames` must be a valid array of `fcount` non-null C string pointers.
#[no_mangle]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_mkspell(
    fcount: c_int,
    fnames: *mut *mut c_char,
    ascii: bool,
    over_write: bool,
    added_word: bool,
) {
    const MAXREGIONS: c_int = 8;
    const MAXPATHL: usize = 4096;

    // Initialize spellinfo_T to all-zeros.
    let mut spin: SpellinfoT = std::mem::zeroed();
    spin.si_verbose = c_int::from(!added_word);
    spin.si_ascii = c_int::from(ascii);
    spin.si_followup = 1;
    spin.si_rem_accents = 1;
    ga_init_wvs(
        &raw mut spin.si_rep,
        std::mem::size_of::<FromtoC>() as c_int,
        20,
    );
    ga_init_wvs(
        &raw mut spin.si_repsal,
        std::mem::size_of::<FromtoC>() as c_int,
        20,
    );
    ga_init_wvs(
        &raw mut spin.si_sal,
        std::mem::size_of::<FromtoC>() as c_int,
        20,
    );
    ga_init_wvs(
        &raw mut spin.si_map,
        std::mem::size_of::<c_char>() as c_int,
        100,
    );
    ga_init_wvs(
        &raw mut spin.si_comppat,
        std::mem::size_of::<*mut c_char>() as c_int,
        20,
    );
    ga_init_wvs(
        &raw mut spin.si_prefcond,
        std::mem::size_of::<*mut c_char>() as c_int,
        50,
    );
    hash_init_wvs(&raw mut spin.si_commonwords);
    spin.si_newcompID = 127;

    // Default: fnames[0] is output, rest are inputs.
    let innames: *mut *mut c_char = if fcount == 1 { fnames } else { fnames.add(1) };
    let mut incount: c_int = fcount - 1;

    let wfname = xmalloc_wvs(MAXPATHL).cast::<c_char>();

    // Compute output filename via Rust helper.
    if fcount >= 1 {
        let enc_str = if spin.si_ascii != 0 {
            c"ascii".as_ptr().cast::<u8>()
        } else {
            spell_enc().cast::<u8>()
        };
        let mut fname_res: MkspellFnameResult = std::mem::zeroed();
        let fret = rs_mkspell_output_fname(
            fnames.cast::<*const u8>().cast_const(),
            fcount,
            enc_str,
            &raw mut fname_res,
        );
        if fret == 0 && fname_res.fname_len > 0 {
            xstrlcpy_wvs(wfname, fname_res.fname.as_ptr().cast::<c_char>(), MAXPATHL);
            if fname_res.is_ascii {
                spin.si_ascii = 1;
            }
            if fname_res.is_add {
                spin.si_add = 1;
            }
            if fcount == 1 {
                incount = 1;
            }
        }
    }

    if incount <= 0 {
        emsg_wvs(E_INVARG_WVS.as_ptr().cast::<c_char>());
        xfree_wvs(wfname.cast::<c_void>());
        return;
    }
    if !vim_strchr_wvs(path_tail_wvs(wfname), b'_' as c_int).is_null() {
        emsg_wvs(E_REGION_WVS.as_ptr().cast::<c_char>());
        xfree_wvs(wfname.cast::<c_void>());
        return;
    }
    if incount > MAXREGIONS {
        semsg_wvs(E_TOOMANY_WVS.as_ptr().cast::<c_char>(), MAXREGIONS);
        xfree_wvs(wfname.cast::<c_void>());
        return;
    }

    // Check for overwriting before doing expensive work.
    if !over_write && os_path_exists(wfname) {
        // e_exists: "E739: Cannot create directory" is wrong for this case.
        // Use the correct "E739: File exists" message which is what C uses.
        // In C: emsg(_(e_exists)) which is E739.
        emsg_wvs(E_EXISTS_WVS.as_ptr().cast::<c_char>());
        xfree_wvs(wfname.cast::<c_void>());
        return;
    }
    if os_isdir(wfname) {
        semsg_wvs(E_ISADIR2_WVS.as_ptr().cast::<c_char>(), wfname);
        xfree_wvs(wfname.cast::<c_void>());
        return;
    }

    let fname = xmalloc_wvs(MAXPATHL).cast::<c_char>();
    let mut error = false;
    let mut afile: [*mut AfffileT; 8] = [std::ptr::null_mut(); 8];

    // Validate input filenames and extract region names for multi-region builds.
    if incount > 1 {
        let vret = rs_mkspell_validate_args(
            innames.cast::<*const u8>().cast_const(),
            incount,
            spin.si_region_name.as_mut_ptr().cast::<u8>(),
        );
        if vret == 1 {
            // Find offending filename and report it.
            for i in 0..incount as usize {
                let name = *innames.add(i);
                let len = libc::strlen(name);
                let tail = path_tail_wvs(name);
                let tail_len = libc::strlen(tail);
                if tail_len < 5 || *name.add(len - 3) != b'_' as c_char {
                    semsg_wvs(E_INVREGION_WVS.as_ptr().cast::<c_char>(), name);
                    xfree_wvs(fname.cast::<c_void>());
                    xfree_wvs(wfname.cast::<c_void>());
                    return;
                }
            }
            xfree_wvs(fname.cast::<c_void>());
            xfree_wvs(wfname.cast::<c_void>());
            return;
        }
    }

    spin.si_region_count = incount;

    spin.si_foldroot = rs_wordtree_alloc(&raw mut spin);
    spin.si_keeproot = rs_wordtree_alloc(&raw mut spin);
    spin.si_prefroot = rs_wordtree_alloc(&raw mut spin);

    if spin.si_add == 0 {
        spin.si_clear_chartab = 1;
    }

    // Read all .aff and .dic files (text is converted to 'encoding').
    for (i, af) in afile[..incount as usize].iter_mut().enumerate() {
        (*spin_conv(&raw mut spin)).vc_type = CONV_NONE;
        spin.si_region = 1 << i;
        let inname = *innames.add(i);

        vim_snprintf_wvs(fname, MAXPATHL, c"%s.aff".as_ptr(), inname);
        if os_path_exists(fname) {
            *af = rs_spell_read_aff(&raw mut spin, fname);
            if af.is_null() {
                error = true;
            } else {
                vim_snprintf_wvs(fname, MAXPATHL, c"%s.dic".as_ptr(), inname);
                if rs_spell_read_dic(&raw mut spin, fname, *af) != OK {
                    error = true;
                }
            }
        } else if rs_spell_read_wordfile(&raw mut spin, inname) != OK {
            error = true;
        }

        convert_setup_wvs(
            spin_conv(&raw mut spin),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
    }

    if !spin.si_compflags.is_null() && spin.si_nobreak != 0 {
        msg_wvs(MSG_COMPOUND_NOBREAK_WVS.as_ptr().cast::<c_char>(), 0);
    }

    if !error && !got_int_wvs {
        // Compress word trees.
        rs_spell_message(&raw mut spin, MSG_COMPRESS_WVS.as_ptr().cast::<c_char>());
        rs_wordtree_compress_export(&raw mut spin, spin.si_foldroot, c"case-folded".as_ptr());
        rs_wordtree_compress_export(&raw mut spin, spin.si_keeproot, c"keep-case".as_ptr());
        rs_wordtree_compress_export(&raw mut spin, spin.si_prefroot, c"prefixes".as_ptr());
    }

    if !error && !got_int_wvs {
        let iobuff = std::ptr::addr_of_mut!(IObuff_wvs).cast::<c_char>();
        vim_snprintf_wvs(
            iobuff,
            1025,
            MSG_WRITING_WVS.as_ptr().cast::<c_char>(),
            wfname,
        );
        rs_spell_message(&raw mut spin, iobuff);

        error = rs_write_vim_spell(&raw mut spin, wfname) == FAIL;

        rs_spell_message(&raw mut spin, MSG_DONE_WVS.as_ptr().cast::<c_char>());
        let iobuff2 = std::ptr::addr_of_mut!(IObuff_wvs).cast::<c_char>();
        vim_snprintf_wvs(
            iobuff2,
            1025,
            MSG_MEMUSE_WVS.as_ptr().cast::<c_char>(),
            spin.si_memtot,
        );
        rs_spell_message(&raw mut spin, iobuff2);

        if !error {
            rs_spell_reload_one(wfname, added_word);
        }
    }

    // Free allocated memory.
    ga_clear_wvs(&raw mut spin.si_rep);
    ga_clear_wvs(&raw mut spin.si_repsal);
    ga_clear_wvs(&raw mut spin.si_sal);
    ga_clear_wvs(&raw mut spin.si_map);
    ga_clear_wvs(&raw mut spin.si_comppat);
    ga_clear_wvs(&raw mut spin.si_prefcond);
    hash_clear_all_wvs(&raw mut spin.si_commonwords, 0);

    for af in &afile[..incount as usize] {
        if !af.is_null() {
            rs_spell_free_aff(*af);
        }
    }

    rs_free_blocks(spin.si_blocks);

    // Create .sug file if soundfolding info was written.
    if spin.si_sugtime != 0 && !error && !got_int_wvs {
        rs_spell_make_sugfile(&raw mut spin, wfname);
    }

    xfree_wvs(fname.cast::<c_void>());
    xfree_wvs(wfname.cast::<c_void>());
}

// =============================================================================
// spell_add_word and init_spellfile (Phase 3 migration)
// =============================================================================

extern "C" {
    // Globals and accessors (spell_shim.c Phase 3)
    fn nvim_get_int_wordlist() -> *mut c_char;
    fn nvim_set_int_wordlist(val: *mut c_char);
    fn nvim_curwin_get_ws_b_p_spf() -> *mut c_char;
    fn nvim_curwin_get_ws_b_p_spl() -> *mut c_char;
    fn nvim_curwin_ws_b_langp_is_empty() -> bool;
    fn nvim_curwin_get_ws_b_langp() -> *const crate::GArrayRaw;
    fn nvim_curbuf_get_b_s_b_p_spl() -> *mut c_char;
    fn nvim_get_NameBuff() -> *mut c_char;
    fn nvim_buf_ml_mfp_is_null(buf: *mut c_void) -> bool;

    // String / path functions
    fn vim_tempname() -> *mut c_char;
    fn copy_option_part(
        str: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep_chars: *const c_char,
    ) -> usize;
    fn home_replace(
        buf: *const c_void,
        src: *const c_char,
        dst: *mut c_char,
        dstlen: usize,
        one: bool,
    );
    fn dir_of_file_exists(fname: *const c_char) -> bool;
    #[link_name = "path_tail_with_sep"]
    fn path_tail_with_sep_saw(fname: *mut c_char) -> *mut c_char;
    #[link_name = "os_mkdir"]
    fn os_mkdir_saw(path: *const c_char, perm: u32) -> c_int;
    fn vim_ispathsep(c: c_int) -> bool;
    #[link_name = "vim_strchr"]
    fn vim_strchr_saw(str: *const c_char, c: c_int) -> *mut c_char;
    fn get_xdg_home(xdg: c_int) -> *mut c_char;
    fn xstrlcat(dst: *mut c_char, src: *const c_char, dstsize: usize) -> usize;
    fn os_mkdir_recurse(
        path: *const c_char,
        perm: u32,
        failed_dir: *mut *mut c_char,
        created_dir: *mut *mut c_char,
    ) -> c_int;

    // Buffer functions (use shim names - buflist_findname_exp is a C inline)
    fn rs_buflist_findname_exp(fname: *mut c_char) -> *mut c_void;
    fn nvim_buf_is_changed(buf: *mut c_void) -> c_int;
    #[link_name = "buf_reload"]
    fn nvim_buf_reload(buf: *mut c_void, orig_mode: c_int, reload_options: c_int);

    // File I/O (use libc via Rust's libc crate for portability)
    #[link_name = "os_fopen"]
    fn os_fopen_saw(fname: *const c_char, mode: *const c_char) -> *mut libc::FILE;

    // Memory
    #[link_name = "xmalloc"]
    fn xmalloc_saw(size: usize) -> *mut c_void;
    #[link_name = "xfree"]
    fn xfree_saw(ptr: *mut c_void);

    // Messaging
    #[link_name = "emsg"]
    fn emsg_saw(s: *const c_char) -> bool;
    #[link_name = "semsg"]
    fn semsg_saw(fmt: *const c_char, ...) -> bool;
    #[link_name = "smsg"]
    fn smsg_saw(hl_id: c_int, fmt: *const c_char, ...) -> c_int;

    // Validation
    fn valid_spell_word(word: *const c_char, end: *const c_char) -> bool;

    // Redraw
    #[link_name = "redraw_all_later"]
    fn redraw_all_later_saw(type_: c_int);
}

// kXDGDataHome = 0 (from stdpaths_defs.h)
const K_XDG_DATA_HOME: c_int = 0;

// UPD_SOME_VALID is already defined at the top of this file (= 35).

/// E_NOTSET error message key
const E_NOTSET: *const c_char = c"E764: Option '%s' is not set".as_ptr();
/// E_NOTOPEN error message key
const E_NOTOPEN: *const c_char = c"E484: Can't open file %s".as_ptr();
/// E_BUFLOADED error message
const E_BUFLOADED: *const c_char = c"E139: File is loaded in another buffer".as_ptr();
/// E_ILLEGAL_CHAR_IN_WORD
const E_ILLEGAL_CHAR: *const c_char = c"E1280: Illegal character in word".as_ptr();

const MAXPATHL_SAW: usize = 4096;

// Use the existing rs_spell_add_word_format and rs_spell_find_duplicate_word
// as direct Rust function calls (they're defined in this module).

/// Initialize the 'spellfile' option for the current window/buffer.
///
/// If the location does not exist, create it. Defaults to
/// stdpath("data") + "/site/spell/{spelllang}.{encoding}.add".
///
/// # Safety
/// Must be called from main thread with valid curwin/curbuf state.
#[allow(clippy::cast_possible_wrap)]
unsafe fn init_spellfile() {
    let mut aspath = false;
    let mut lstart = nvim_curbuf_get_b_s_b_p_spl();

    let b_p_spl = nvim_curwin_get_ws_b_p_spl();
    if *b_p_spl == 0 || nvim_curwin_ws_b_langp_is_empty() {
        return;
    }

    // Find the end of the language name; exclude the region.
    // If there is a path separator, remember the start of the tail.
    let mut lend = b_p_spl;
    loop {
        let ch = *lend;
        if ch == 0 {
            break;
        }
        // Check vim_strchr(",._", ch)
        let in_sep = vim_strchr_saw(c",._".as_ptr(), ch as c_int);
        if !in_sep.is_null() {
            break;
        }
        if vim_ispathsep(ch as c_int) {
            aspath = true;
            lstart = lend.add(1);
        }
        lend = lend.add(1);
    }

    let buf = xmalloc_saw(MAXPATHL_SAW).cast::<c_char>();
    let buf_len = MAXPATHL_SAW;

    if aspath {
        let lstart_spl = nvim_curbuf_get_b_s_b_p_spl();
        let copy_len = (lend as usize).wrapping_sub(lstart_spl as usize);
        if copy_len >= buf_len {
            xfree_saw(buf.cast::<c_void>());
            return;
        }
        // xmemcpyz equivalent: copy + NUL-terminate
        std::ptr::copy_nonoverlapping(lstart_spl, buf, copy_len);
        *buf.add(copy_len) = 0;
    } else {
        // Use XDG data home
        let xdg_path = get_xdg_home(K_XDG_DATA_HOME);
        xstrlcpy(buf, xdg_path, buf_len);
        xfree_saw(xdg_path.cast::<c_void>());

        xstrlcat(buf, c"/site/spell".as_ptr(), buf_len);

        let mut failed_dir: *mut c_char = std::ptr::null_mut();
        if os_mkdir_recurse(buf, 0o755, &raw mut failed_dir, std::ptr::null_mut()) != 0 {
            xfree_saw(buf.cast::<c_void>());
            xfree_saw(failed_dir.cast::<c_void>());
            return;
        }
    }

    // Append spelllang tail
    let lend_usize = lend as usize;
    let lstart_usize = lstart as usize;
    let tail_len = lend_usize.wrapping_sub(lstart_usize);
    let buf_used = libc::strlen(buf.cast::<libc::c_char>());
    // Append "/" + lstart..lend
    if buf_used + 1 + tail_len + 1 < buf_len {
        let p = buf.add(buf_used);
        *p = 47; // b'/'
        std::ptr::copy_nonoverlapping(lstart, p.add(1), tail_len);
        *p.add(1 + tail_len) = 0;
    }

    // Append ".ascii.add" or ".{enc}.add"
    let langp_ga = nvim_curwin_get_ws_b_langp();
    let lp = crate::langp_entry(langp_ga, 0);
    let slang = (*lp).lp_slang;
    let sl_fname = if slang.is_null() {
        std::ptr::null_mut()
    } else {
        (*slang).sl_fname
    };
    let enc_suffix: *const c_char = if sl_fname.is_null() {
        spell_enc()
    } else {
        let tail = path_tail(sl_fname);
        // Check if tail contains ".ascii."
        let ascii_str = c".ascii.".as_ptr();
        if libc::strstr(
            tail.cast::<libc::c_char>(),
            ascii_str.cast::<libc::c_char>(),
        )
        .is_null()
        {
            spell_enc()
        } else {
            c"ascii".as_ptr()
        }
    };

    // Append ".{enc}.add"
    let buf_used2 = libc::strlen(buf.cast::<libc::c_char>());
    // Format: ".{enc}.add"
    let enc_len = libc::strlen(enc_suffix.cast::<libc::c_char>());
    if buf_used2 + 1 + enc_len + 4 + 1 < buf_len {
        let p = buf.add(buf_used2);
        *p = 46; // b'.'
        std::ptr::copy_nonoverlapping(enc_suffix, p.add(1), enc_len);
        let q = p.add(1 + enc_len);
        *q = 46; // b'.'
        *q.add(1) = 97; // b'a'
        *q.add(2) = 100; // b'd'
        *q.add(3) = 100; // b'd'
        *q.add(4) = 0;
    }

    nvim_spell_set_spellfile_option(buf);
    xfree_saw(buf.cast::<c_void>());
}

extern "C" {
    /// Thin shim: set_option_value_give_err(kOptSpellfile, CSTR_AS_OPTVAL(buf), OPT_LOCAL)
    fn nvim_spell_set_spellfile_option(buf: *const c_char);
}

/// Add "word[len]" to 'spellfile' as a good or bad word.
///
/// # Safety
/// Must be called from main thread with valid Neovim state.
#[export_name = "spell_add_word"]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_spell_add_word(
    word: *mut c_char,
    len: c_int,
    what: c_int,
    idx: c_int,
    undo: bool,
) {
    // SPELL_ADD_* values
    const SPELL_ADD_BAD: c_int = 1;

    let mut buf_ptr: *mut c_void = std::ptr::null_mut(); // buf_T*
    let mut new_spf = false;
    let mut file_written = false;
    let mut fname: *mut c_char;
    let mut fnamebuf: *mut c_char = std::ptr::null_mut();

    if !valid_spell_word(word, word.add(len as usize)) {
        let _ = emsg_saw(E_ILLEGAL_CHAR);
        return;
    }

    if idx == 0 {
        // Use internal word list
        if nvim_get_int_wordlist().is_null() {
            let tmp = vim_tempname();
            if tmp.is_null() {
                return;
            }
            nvim_set_int_wordlist(tmp);
        }
        fname = nvim_get_int_wordlist();
    } else {
        // If 'spellfile' isn't set, figure out a default.
        if *nvim_curwin_get_ws_b_p_spf() == 0 {
            init_spellfile();
            new_spf = true;
        }

        if *nvim_curwin_get_ws_b_p_spf() == 0 {
            let _ = semsg_saw(E_NOTSET, c"spellfile".as_ptr());
            return;
        }

        fnamebuf = xmalloc_saw(MAXPATHL_SAW).cast::<c_char>();
        let mut spf = nvim_curwin_get_ws_b_p_spf();
        let mut i = 1i32;
        loop {
            if *spf == 0 {
                break;
            }
            copy_option_part(&raw mut spf, fnamebuf, MAXPATHL_SAW, c",".as_ptr());
            if i == idx {
                break;
            }
            if *spf == 0 {
                let _ = semsg_saw(
                    c"E765: 'spellfile' does not have %ld entries".as_ptr(),
                    idx as i64,
                );
                xfree_saw(fnamebuf.cast::<c_void>());
                return;
            }
            i += 1;
        }

        // Check that the user isn't editing the .add file somewhere.
        buf_ptr = rs_buflist_findname_exp(fnamebuf);
        if !buf_ptr.is_null() && nvim_buf_ml_mfp_is_null(buf_ptr) {
            buf_ptr = std::ptr::null_mut();
        }
        if !buf_ptr.is_null() && nvim_buf_is_changed(buf_ptr) != 0 {
            let _ = emsg_saw(E_BUFLOADED);
            xfree_saw(fnamebuf.cast::<c_void>());
            return;
        }

        fname = fnamebuf;
    }

    if what == SPELL_ADD_BAD || undo {
        // Read the whole file to find duplicates.
        let fd = os_fopen_saw(fname, c"r".as_ptr());
        if !fd.is_null() {
            libc::fseek(fd, 0, libc::SEEK_END);
            let fsize = libc::ftell(fd);
            libc::rewind(fd);

            if fsize > 0 {
                let fbuf = xmalloc_saw(fsize as usize).cast::<u8>();
                let nread = libc::fread(fbuf.cast::<c_void>(), 1, fsize as usize, fd);
                libc::fclose(fd);

                let mut scan_offset = 0usize;
                while scan_offset < nread {
                    let mut match_offset = 0usize;
                    if !rs_spell_find_duplicate_word(
                        fbuf.add(scan_offset),
                        nread - scan_offset,
                        word.cast::<u8>(),
                        len as usize,
                        &raw mut match_offset,
                    ) {
                        break;
                    }
                    let abs_offset = scan_offset + match_offset;

                    // Comment out the line by writing '#'.
                    let fd2 = os_fopen_saw(fname, c"r+".as_ptr());
                    if !fd2.is_null() {
                        if libc::fseek(fd2, abs_offset as libc::c_long, libc::SEEK_SET) == 0 {
                            libc::fputc(b'#' as libc::c_int, fd2);
                            file_written = true;
                            if undo {
                                let name_buff = nvim_get_NameBuff();
                                home_replace(
                                    std::ptr::null(),
                                    fname,
                                    name_buff,
                                    MAXPATHL_SAW,
                                    true,
                                );
                                let _ = smsg_saw(
                                    0,
                                    c"Word '%.*s' removed from %s".as_ptr(),
                                    len,
                                    word,
                                    name_buff,
                                );
                            }
                        }
                        libc::fclose(fd2);
                    }

                    // Advance past this line.
                    let mut next = abs_offset;
                    let content = std::slice::from_raw_parts(fbuf, nread);
                    while next < nread && content[next] != b'\n' {
                        next += 1;
                    }
                    scan_offset = next + 1;
                }
                xfree_saw(fbuf.cast::<c_void>());
            } else {
                libc::fclose(fd);
            }
        }
    }

    if !undo {
        let mut fd = os_fopen_saw(fname, c"a".as_ptr());
        if fd.is_null() && new_spf {
            // May need to create the "spell" directory first.
            if !dir_of_file_exists(fname) {
                let p = path_tail_with_sep_saw(fname);
                if p != fname {
                    let c = *p;
                    *p = 0;
                    os_mkdir_saw(fname, 0o755);
                    *p = c;
                    fd = os_fopen_saw(fname, c"a".as_ptr());
                }
            }
        }

        if fd.is_null() {
            let _ = semsg_saw(E_NOTOPEN, fname);
        } else {
            // Format the line to append.
            let mut append_buf = [0u8; MAXWLEN * 2 + 4];
            let append_len = rs_spell_add_word_format(
                word.cast::<u8>(),
                len as usize,
                what,
                append_buf.as_mut_ptr(),
                append_buf.len(),
            );
            if append_len > 0 {
                libc::fwrite(
                    append_buf.as_ptr().cast::<c_void>(),
                    1,
                    append_len as usize,
                    fd,
                );
                file_written = true;
            }
            libc::fclose(fd);

            let name_buff = nvim_get_NameBuff();
            home_replace(std::ptr::null(), fname, name_buff, MAXPATHL_SAW, true);
            let _ = smsg_saw(0, c"Word '%.*s' added to %s".as_ptr(), len, word, name_buff);
        }
    }

    if file_written {
        // Update the .add.spl file.
        rs_mkspell(1, &raw mut fname, false, true, true);

        // Reload if edited elsewhere.
        if !buf_ptr.is_null() {
            let orig_mode = (*buf_ptr.cast::<nvim_buffer::buf_struct::BufStruct>()).b_orig_mode;
            nvim_buf_reload(buf_ptr, orig_mode, 0);
        }

        redraw_all_later_saw(UPD_SOME_VALID);
    }
    xfree_saw(fnamebuf.cast::<c_void>());
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
