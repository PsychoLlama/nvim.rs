//! Included ranges: the sub-spans of a source a parser is restricted to.
//!
//! A range is a byte span plus its two [`TSPoint`]s, and it crosses the Lua
//! boundary as a 4- or a 6-element list: [`push_ranges`] renders either
//! shape, `range_from_lua` reads the 6-element one (or a node's range).
//! `set_included_ranges` and `included_ranges` are the parser methods over
//! them.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/treesitter/` row in docs/perimeter.md.
#![allow(unsafe_code)]

use core::ffi::{c_int, c_void};

use super::node::check_node;
use super::parser::check_parser;
use super::*;
use crate::lua::stack::{Kind, Raise};
use crate::memory::xfree;

/// Push `ranges` as a list of lists: `{start_row, start_col, end_row,
/// end_col}`, with the two byte offsets after each point's pair when
/// `include_bytes`.
pub(crate) fn push_ranges(stack: &mut Stack, ranges: &[TSRange], include_bytes: bool) {
    stack.new_table(ranges.len(), 0);
    for (index, range) in (1..).zip(ranges) {
        let fields: &[u32] = if include_bytes {
            &[
                range.start_point.row,
                range.start_point.column,
                range.start_byte,
                range.end_point.row,
                range.end_point.column,
                range.end_byte,
            ]
        } else {
            &[
                range.start_point.row,
                range.start_point.column,
                range.end_point.row,
                range.end_point.column,
            ]
        };
        stack.new_table(fields.len(), 0);
        for (slot, &field) in (1..).zip(fields) {
            stack.push_number(f64::from(field));
            stack.raw_set_index(At::from_top(2), slot);
        }
        stack.raw_set_index(At::from_top(2), index);
    }
}

/// [`push_ranges`] over an array tree-sitter allocated for the caller, which
/// is freed after.
///
/// # Safety
/// `ranges` points at `length` ranges allocated through the editor's
/// allocator (`ts_set_allocator`) and handed over, or is null with no
/// length.
pub(crate) unsafe fn push_owned_ranges(
    stack: &mut Stack,
    ranges: *mut TSRange,
    length: u32,
    include_bytes: bool,
) {
    // SAFETY: the caller's array, read before it is freed.
    unsafe {
        push_ranges(stack, range_slice(ranges, length), include_bytes);
        xfree(ranges.cast::<c_void>());
    }
}

/// The `length` ranges at `ranges` as a slice; empty for null.
///
/// # Safety
/// `ranges` points at `length` live ranges for `'a`, or is null.
pub(crate) unsafe fn range_slice<'a>(ranges: *const TSRange, length: u32) -> &'a [TSRange] {
    if ranges.is_null() {
        return &[];
    }
    // SAFETY: the caller's array.
    unsafe { core::slice::from_raw_parts(ranges, length as usize) }
}

/// The error for a range that is neither shape.
fn range_error() -> Raise {
    Raise::message("Ranges can only be made from 6 element long tables or nodes.")
}

/// The number on top as a `uint32_t`, refusing what does not fit.
fn check_uint32(stack: &Stack) -> Result<u32, Raise> {
    let value = stack.check_number(At::TOP)?;
    // `(uint32_t)value`; checked against `value` below, so a value it does
    // not hold is refused rather than used.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let converted = value as u32;
    if value < 0.0 || value > f64::from(u32::MAX) || f64::from(converted) != value {
        return Err(Raise::message("Range value out of bounds"));
    }
    Ok(converted)
}

/// The range the value on top stands for: a 6-element list, or a node.
fn range_from_lua(stack: &mut Stack) -> Result<TSRange, Raise> {
    if stack.kind(At::TOP) != Kind::Table {
        // `luaL_checkudata` refuses anything but a node, numbering it -1.
        let node = check_node(stack, At::TOP)?;
        // SAFETY: a node of a tree its userdatum keeps alive.
        return Ok(unsafe {
            TSRange {
                start_point: ts_node_start_point(node),
                end_point: ts_node_end_point(node),
                start_byte: ts_node_start_byte(node),
                end_byte: ts_node_end_byte(node),
            }
        });
    }
    if stack.raw_len(At::TOP) != 6 {
        return Err(range_error());
    }
    let mut fields = [0; 6];
    for (index, field) in (1..).zip(&mut fields) {
        stack.raw_get_index(At::TOP, index);
        // A refusal leaves the element on top: the check's error names it.
        *field = check_uint32(stack)?;
        stack.pop(1);
    }
    let [start_row, start_col, start_byte, end_row, end_col, end_byte] = fields;
    Ok(TSRange {
        start_point: TSPoint {
            row: start_row,
            column: start_col,
        },
        end_point: TSPoint {
            row: end_row,
            column: end_col,
        },
        start_byte,
        end_byte,
    })
}

/// `parser:set_included_ranges(ranges)`.
pub(crate) fn parser_set_ranges(stack: &mut Stack) -> Result<c_int, Raise> {
    if stack.top().index() < 2 {
        return Err(Raise::message(
            "not enough args to parser:set_included_ranges()",
        ));
    }
    let parser = check_parser(stack, At::arg(1))?;
    if stack.kind(At::arg(2)) != Kind::Table {
        return Err(Raise::argument(2, "table expected."));
    }
    let count = stack.raw_len(At::arg(2));
    let mut ranges = Vec::with_capacity(count);
    for index in 1..=c_int::try_from(count).unwrap_or(c_int::MAX) {
        stack.raw_get_index(At::arg(2), index);
        ranges.push(range_from_lua(stack)?);
        stack.pop(1);
    }
    let count = u32::try_from(ranges.len()).unwrap_or(u32::MAX);
    // SAFETY: the parser the userdatum in slot 1 owns, and `count` ranges
    // it copies.
    unsafe { ts_parser_set_included_ranges(parser, ranges.as_ptr(), count) };
    Ok(0)
}

/// `parser:included_ranges([include_bytes])`.
pub(crate) fn parser_get_ranges(stack: &mut Stack) -> Result<c_int, Raise> {
    let parser = check_parser(stack, At::arg(1))?;
    let include_bytes = stack.top().index() >= 2 && stack.to_boolean(At::arg(2));
    let mut length: u32 = 0;
    // SAFETY: as above; the array is the parser's own, read before
    // anything can change it.
    unsafe {
        let ranges = ts_parser_included_ranges(parser, &raw mut length);
        push_ranges(stack, range_slice(ranges, length), include_bytes);
    }
    Ok(1)
}
