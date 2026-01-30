Plan: ~/.claude/plans/nifty-cooking-axolotl.md

Let me provide a summary of the completed work.

## Summary

I've completed the migration of fillchars/listchars parsing to Rust according to the plan. Here's what was accomplished:

### Phase 1: Migrate `get_encoded_char_adv()` ✅
- Created `rs_get_encoded_char_adv()` in Rust that handles escape sequences (`\xNN`, `\uNNNN`, `\UNNNNNNNN`)
- Added extern declarations for `rs_hexhex2nr`, `rs_char2cells`, `rs_utfc_ptr2len`, `rs_utfc_ptr2schar`, `rs_schar_from_char`
- Updated C function to delegate to Rust implementation

### Phase 2: Migrate parsing infrastructure ✅
- Created `CharsFieldResult` struct and `CharsParseError` enum
- Implemented `rs_parse_chars_field()` - parses a single field:value pair
- Implemented `rs_count_multispace_chars()` and `rs_parse_multispace_chars()` for multispace fields
- Added field index constants and `find_field_index()` helper

### Phase 3: Migrate validation wrapper ✅
- Created `CharsValidateResult` struct with detailed error info
- Implemented `rs_validate_chars_option()` - validates complete option value
- Implemented `rs_is_valid_chars_option()` - simple bool validation

### Phase 4: Integration and cleanup ✅
- All success criteria verified:
  - `:set fillchars=vert:\|,fold:-` works
  - `:set listchars=tab:>-,trail:·` works
  - `:set fillchars=invalid` shows error
- All checks pass: `just build`, `just rust-fmt-check`, `just rust-clippy`, `just rust-test`

### Commits Created
1. `ad0d51f9dd` - feat(optionstr): migrate get_encoded_char_adv() to Rust
2. `80f3a30103` - feat(optionstr): add chars field parsing infrastructure  
3. `5d37a84eaf` - feat(optionstr): add chars option validation wrapper
