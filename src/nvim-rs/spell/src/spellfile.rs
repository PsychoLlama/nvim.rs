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
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
}
