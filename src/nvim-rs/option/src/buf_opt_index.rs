//! BufOptIndex constants for Neovim buffer-local options
//!
//! These constants mirror the C `BufOptIndex` enum values from
//! `build/src/nvim/auto/options_enum.generated.h`.
//!
//! They are kept in sync with the generated C enum. If a buffer-local option
//! is added or reordered in options.lua, both the C enum and these constants
//! will change; a compile-time _Static_assert in option_shim.c validates
//! the mapping.

use std::ffi::c_int;

pub const K_BUF_OPT_INVALID: c_int = -1;
pub const K_BUF_OPT_AUTOCOMPLETE: c_int = 0;
pub const K_BUF_OPT_AUTOINDENT: c_int = 1;
pub const K_BUF_OPT_AUTOREAD: c_int = 2;
pub const K_BUF_OPT_BACKUPCOPY: c_int = 3;
pub const K_BUF_OPT_BINARY: c_int = 4;
pub const K_BUF_OPT_BOMB: c_int = 5;
pub const K_BUF_OPT_BUFHIDDEN: c_int = 6;
pub const K_BUF_OPT_BUFLISTED: c_int = 7;
pub const K_BUF_OPT_BUFTYPE: c_int = 8;
pub const K_BUF_OPT_BUSY: c_int = 9;
pub const K_BUF_OPT_CHANNEL: c_int = 10;
pub const K_BUF_OPT_CINDENT: c_int = 11;
pub const K_BUF_OPT_CINKEYS: c_int = 12;
pub const K_BUF_OPT_CINOPTIONS: c_int = 13;
pub const K_BUF_OPT_CINSCOPEDECLS: c_int = 14;
pub const K_BUF_OPT_CINWORDS: c_int = 15;
pub const K_BUF_OPT_COMMENTS: c_int = 16;
pub const K_BUF_OPT_COMMENTSTRING: c_int = 17;
pub const K_BUF_OPT_COMPLETE: c_int = 18;
pub const K_BUF_OPT_COMPLETEFUNC: c_int = 19;
pub const K_BUF_OPT_COMPLETEOPT: c_int = 20;
pub const K_BUF_OPT_COMPLETESLASH: c_int = 21;
pub const K_BUF_OPT_COPYINDENT: c_int = 22;
pub const K_BUF_OPT_DEFINE: c_int = 23;
pub const K_BUF_OPT_DICTIONARY: c_int = 24;
pub const K_BUF_OPT_DIFFANCHORS: c_int = 25;
pub const K_BUF_OPT_ENDOFFILE: c_int = 26;
pub const K_BUF_OPT_ENDOFLINE: c_int = 27;
pub const K_BUF_OPT_EQUALPRG: c_int = 28;
pub const K_BUF_OPT_ERRORFORMAT: c_int = 29;
pub const K_BUF_OPT_EXPANDTAB: c_int = 30;
pub const K_BUF_OPT_FILEENCODING: c_int = 31;
pub const K_BUF_OPT_FILEFORMAT: c_int = 32;
pub const K_BUF_OPT_FILETYPE: c_int = 33;
pub const K_BUF_OPT_FINDFUNC: c_int = 34;
pub const K_BUF_OPT_FIXENDOFLINE: c_int = 35;
pub const K_BUF_OPT_FORMATEXPR: c_int = 36;
pub const K_BUF_OPT_FORMATLISTPAT: c_int = 37;
pub const K_BUF_OPT_FORMATOPTIONS: c_int = 38;
pub const K_BUF_OPT_FORMATPRG: c_int = 39;
pub const K_BUF_OPT_GREPFORMAT: c_int = 40;
pub const K_BUF_OPT_GREPPRG: c_int = 41;
pub const K_BUF_OPT_IMINSERT: c_int = 42;
pub const K_BUF_OPT_IMSEARCH: c_int = 43;
pub const K_BUF_OPT_INCLUDE: c_int = 44;
pub const K_BUF_OPT_INCLUDEEXPR: c_int = 45;
pub const K_BUF_OPT_INDENTEXPR: c_int = 46;
pub const K_BUF_OPT_INDENTKEYS: c_int = 47;
pub const K_BUF_OPT_INFERCASE: c_int = 48;
pub const K_BUF_OPT_ISKEYWORD: c_int = 49;
pub const K_BUF_OPT_KEYMAP: c_int = 50;
pub const K_BUF_OPT_KEYWORDPRG: c_int = 51;
pub const K_BUF_OPT_LISP: c_int = 52;
pub const K_BUF_OPT_LISPOPTIONS: c_int = 53;
pub const K_BUF_OPT_LISPWORDS: c_int = 54;
pub const K_BUF_OPT_MAKEENCODING: c_int = 55;
pub const K_BUF_OPT_MAKEPRG: c_int = 56;
pub const K_BUF_OPT_MATCHPAIRS: c_int = 57;
pub const K_BUF_OPT_MODELINE: c_int = 58;
pub const K_BUF_OPT_MODIFIABLE: c_int = 59;
pub const K_BUF_OPT_MODIFIED: c_int = 60;
pub const K_BUF_OPT_NRFORMATS: c_int = 61;
pub const K_BUF_OPT_OMNIFUNC: c_int = 62;
pub const K_BUF_OPT_PATH: c_int = 63;
pub const K_BUF_OPT_PRESERVEINDENT: c_int = 64;
pub const K_BUF_OPT_QUOTEESCAPE: c_int = 65;
pub const K_BUF_OPT_READONLY: c_int = 66;
pub const K_BUF_OPT_SCROLLBACK: c_int = 67;
pub const K_BUF_OPT_SHIFTWIDTH: c_int = 68;
pub const K_BUF_OPT_SMARTINDENT: c_int = 69;
pub const K_BUF_OPT_SOFTTABSTOP: c_int = 70;
pub const K_BUF_OPT_SPELLCAPCHECK: c_int = 71;
pub const K_BUF_OPT_SPELLFILE: c_int = 72;
pub const K_BUF_OPT_SPELLLANG: c_int = 73;
pub const K_BUF_OPT_SPELLOPTIONS: c_int = 74;
pub const K_BUF_OPT_SUFFIXESADD: c_int = 75;
pub const K_BUF_OPT_SWAPFILE: c_int = 76;
pub const K_BUF_OPT_SYNMAXCOL: c_int = 77;
pub const K_BUF_OPT_SYNTAX: c_int = 78;
pub const K_BUF_OPT_TABSTOP: c_int = 79;
pub const K_BUF_OPT_TAGCASE: c_int = 80;
pub const K_BUF_OPT_TAGFUNC: c_int = 81;
pub const K_BUF_OPT_TAGS: c_int = 82;
pub const K_BUF_OPT_TEXTWIDTH: c_int = 83;
pub const K_BUF_OPT_THESAURUS: c_int = 84;
pub const K_BUF_OPT_THESAURUSFUNC: c_int = 85;
pub const K_BUF_OPT_UNDOFILE: c_int = 86;
pub const K_BUF_OPT_UNDOLEVELS: c_int = 87;
pub const K_BUF_OPT_VARSOFTTABSTOP: c_int = 88;
pub const K_BUF_OPT_VARTABSTOP: c_int = 89;
pub const K_BUF_OPT_WRAPMARGIN: c_int = 90;

/// Total number of buffer-local options (kBufOptCount in C).
pub const K_BUF_OPT_COUNT: c_int = 91;
