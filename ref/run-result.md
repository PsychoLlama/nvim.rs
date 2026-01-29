Plan: ~/.claude/plans/noble-wobbling-dove.md

The plan has been executed through Phase Q4. Let me provide a summary of the completed work:

## Summary of Completed Work

### Phase Q1: Errorformat Engine Migration ✅
- Migrated `efm_regpat_bufsz()` and `efm_option_part_len()` to Rust
- Replaced C `efm_to_regpat()` to call `rs_efm_to_regpat()`
- Removed ~160 lines of C code (dead functions: `efmpat_to_regpat`, `scanf_fmt_to_regpat`, `efm_analyze_prefix`, `fmt_pat` array)

### Phase Q2: Line Parsing and Entry Creation ✅
- Added Rust parsing helpers:
  - `rs_qf_parse_prefix_type()` - Extract type from prefix
  - `rs_qf_should_skip_line()` - Check skip flag
  - `rs_qf_is_continuation()` - Check C/Z continuation
  - `rs_qf_starts_multiline()` - Check A/E/W/I/N prefix
  - `rs_qf_is_dir_handler()` - Check D/X prefix
  - `rs_qf_is_file_handler()` - Check O/P/Q prefix
  - `rs_qf_type_is_printable()` - Validate type char
  - `rs_qf_normalize_type()` - Normalize type for storage
- Updated C code to use Rust functions in `qf_parse_line()` and `qf_parse_match()`

### Phase Q3: Initialization and File Reading ✅
- Created new `init.rs` module with:
  - `QfInputSource` enum for input type classification
  - `rs_qf_validate_input_source()` - Validate input
  - `QfReadState` struct for tracking line reading
  - `QfInitOptions` struct for initialization config
  - `rs_qf_init_options_from_action()` - Parse action to options

### Phase Q4: Ex Commands Migration ✅
- Added Ex command helpers:
  - `CcCmdType` enum for :cc/:ll/:crewind/:clast
  - `rs_qf_cc_default_errornr()` - Default error number
  - `rs_qf_cc_is_loclist()` - Check location list variant
  - `CnextCmdType` enum for :cnext/:cprev/:cnfile
  - `rs_qf_cnext_direction()` - Navigation direction
  - `rs_qf_cnext_is_loclist()` - Check location list
  - `rs_qf_cnext_is_file_nav()` - File navigation check

### Statistics
- **Lines in quickfix.c**: 9,747 → 9,598 (reduced by 149 lines)
- **Rust FFI exports**: 520 → 542 (added 22 new exports)
- **Commits**: 7 commits with incremental progress

### Commits Made
1. `cdbf8530ef` - migrate efm_regpat_bufsz and efm_option_part_len to Rust
2. `75ce37c108` - replace efm_to_regpat with Rust implementation
3. `42e93dfb07` - add Phase Q2 line parsing helpers
4. `c011273778` - use Rust parsing helpers in qf_parse_line
5. `5115809bde` - simplify qf_parse_match type assignment
6. `45e1838efe` - add Phase Q3 initialization helpers
7. `08bbc75ebd` - add Phase Q4 Ex command helpers

All changes pass:
- `just build` - Full build with linking
- `just rust-fmt-check` - Formatting
- `just rust-clippy` - Lints
- `just rust-test` - 3669 unit tests

Phases Q5 (API and VimL Functions) and Q6 (Window Management and Display) remain as pending work for future sessions.
