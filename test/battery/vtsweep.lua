-- vtsweep -- the TWENTIETH baselined differential.  Driven by
-- vtsweep.sh, which builds the sandbox and does the scrubs
-- only the shell can see.  Read that header first.
--
-- The subsystem is crates/nvim/src/nvim/vterm/ (18 files, ~7.9k lines):
-- the terminal emulator itself -- parser.rs, state.rs, screen.rs,
-- pen.rs, keyboard.rs, mouse.rs, encoding.rs, csi.rs, dcs.rs, mode.rs,
-- damage.rs, cell.rs, color.rs, geometry.rs, output.rs, selection.rs,
-- text.rs and vterm.rs -- plus the 33 `repr(C)` types in
-- types/vterm{,_internal,_keycodes}.rs that cross the C ABI.
--
-- THE MECHANISM, in one paragraph.  `nvim -u NONE -i NONE -l <this
-- file>` is a Lua script in a started-but-idle editor -- no main loop,
-- no UI, no terminal, no vimrc, no shada; the emulator's thirty
-- `#[no_mangle]` entry points are exported from the binary, so
-- `ffi.C.vterm_*` reaches them directly.  `-ll` (no editor at all) was
-- tried first and ABORTS: `utf_char2cells` reads `p_ambw` on the way to
-- every non-wide character's width and under `-ll` the option pointer
-- is null, so the first leading combining character kills the process
-- (`mbyte/cells.rs:99`).  `vim.o` does not exist under `-ll` and
-- `p_ambw` is not exported, so `-l` is the fix.  The `ffi.cdef`
-- below is SELF-CONTAINED -- it does not read `target/ffi/unit-cdefs.h`
-- and does not need `tools/ffigen` to have run -- which is what makes
-- the sweep runnable against an arbitrary kept binary from months ago.
-- That means the cdef is an INDEPENDENT statement of the ABI, and v0
-- is the assertion that the statement still holds: it prints
-- `ffi.sizeof`/`ffi.alignof` of every type crossing the boundary and
-- `ffi.offsetof` of every field the later sections read.  A layout
-- drift shows up there as a changed number rather than as garbage two
-- thousand rows lower.
--
-- ONE ABI DETAIL IS LOad-BEARING AND MUST NOT BE "TIDIED".  LuaJIT
-- cannot build a callback whose C prototype passes a struct BY VALUE
-- (`ffi.cast('int (*)(VTermRect, void *)', fn)` raises "cannot convert
-- 'function'"), and libvterm's damage, moverect, movecursor, putglyph,
-- erase, scrollrect, osc, dcs, apc, pm, sos and selection-set
-- callbacks all do exactly that.  That is why upstream's own
-- `vterm_spec.lua` needs a compiled C fixture.  This sweep instead
-- declares those callback SLOTS with the System V x86-64 register
-- decomposition of the same signature: a 16-byte `VTermRect` (four
-- ints) is class INTEGER,INTEGER and arrives in two general registers,
-- an 8-byte `VTermPos` in one, and a `VTermStringFragment` (pointer +
-- two uint32) in two.  So `damage(VTermRect, void*)` is declared
-- `int (*)(uint64_t, uint64_t, void*)` and the two words are written
-- back into a scratch buffer and read as the real struct.  The struct's
-- LAYOUT is unchanged (every slot is one function pointer), so v0's
-- sizeof assertions still cover the tables.  This is x86-64 SysV only;
-- on another ABI the decomposition would have to change, and v0's
-- `vt-live` rows are the check that it did not silently rot.
--
-- Sections:
--   v0   canary      sizeof/alignof/offsetof of every crossing type,
--                    plus live probes that prove the cdef describes the
--                    binary actually loaded
--   v1   parser      C0/C1 7- and 8-bit, ESC with intermediates, CSI,
--                    OSC/DCS/APC/PM/SOS, each under four SPLIT
--                    strategies (whole / byte-by-byte / halves / edges)
--   v2   csi         cursor, edit and scroll commands x argument sets
--                    including zero, huge and out-of-range
--   v3   sgr         the pen alphabet incl. 256-colour, 24-bit RGB and
--                    colon sub-parameters, plus DECRQSS read-back
--   v4   modes       DECSET/DECRST, ANSI modes, DECSTBM/DECSLRM,
--                    DECSC/DECRC/DECALN, altscreen 47/1047/1049 and the
--                    kitty key-encoding stack
--   v5   utf8        wide, combining, ZWJ, invalid and truncated UTF-8,
--                    x the utf8 flag, plus the 94/96 charset designators
--   v6   osc         OSC titles, hyperlinks and OSC 52 selection, DCS,
--                    APC/PM/SOS, terminator and fragmentation matrix
--   v7   mouse       X10/UTF8/RXVT/SGR x click/drag/move x modifiers
--   v8   keyboard    every VTermKey and a unichar set x modifiers x
--                    DECCKM/DECKPAM/LNM
--   v9   damage      every VTermDamageSize x scripted paints, dumping
--                    each callback rect AND the residual merge state
--   v10  resize      resize, reflow, scrollback push/pop, altscreen
--   v91  abortprobe  the inputs that may kill the process, one child each
--
-- Every section ends with a `## <name> rows=N` line: a section that
-- goes silently empty otherwise looks exactly like a healthy one.
--
-- DETERMINISM.  Nothing here reads a clock, an address, an environment
-- string or a file.  `vterm_free` is not an exported symbol, so a
-- terminal is never freed and the process leaks ~40 MB by design --
-- which is also why no case may print a pointer.

-- THE JIT IS OFF, AND THAT IS NOT A PERFORMANCE DECISION.  Every
-- by-value struct argument arrives as a pair of `uint64_t`s that this
-- file writes into a scratch array and reads back through a
-- `ffi.cast('VTermRect *', ...)` alias.  LuaJIT's alias analysis does
-- not relate the two cdata types, so a compiled trace forwards the
-- stale load and the ANSWER CHANGES: the `oldpos` of a `movecursor`
-- read `1,4` in one run and `2,4` in the next, from the same binary and
-- the same input.  It reads as flakiness rather than as a bug because
-- LuaJIT 2.1 seeds its hot counters from a PRNG, so WHICH rows are
-- affected moves run to run.  Interpreted, the sweep is byte-identical
-- across runs; it costs about 8x and is worth every bit of it.
local jit = _G.jit
if jit and jit.off then
  jit.off(true, true)
  if jit.flush then
    jit.flush()
  end
end

local ffi = require('ffi')
local bit = require('bit')

ffi.cdef([[
typedef uint32_t ScreenChar;
typedef unsigned int VTermProp;
typedef unsigned int VTermAttr;
typedef unsigned int VTermKey;
typedef unsigned int VTermModifier;
typedef unsigned int VTermDamageSize;
typedef unsigned int VTermTerminator;
typedef unsigned int VTermParserState;
typedef unsigned int VTermSelectionMask;
typedef unsigned int VTermState_mouse_protocol;
typedef unsigned int VTermEncodingType;
typedef int GraphemeState;

typedef struct VTerm VTerm;
typedef struct VTermState VTermState;
typedef struct VTermScreen VTermScreen;
typedef struct VTermEncoding VTermEncoding;

typedef struct VTermPos { int row; int col; } VTermPos;
typedef struct VTermRect { int start_row; int end_row; int start_col; int end_col; } VTermRect;

typedef struct VTermColor_rgb { uint8_t type; uint8_t red; uint8_t green; uint8_t blue; } VTermColor_rgb;
typedef struct VTermColor_indexed { uint8_t type; uint8_t idx; } VTermColor_indexed;
typedef union VTermColor { uint8_t type; VTermColor_rgb rgb; VTermColor_indexed indexed; } VTermColor;

typedef struct VTermStringFragment {
  const char *str;
  uint32_t len : 30;
  bool initial : 1;
  bool final : 1;
  VTermTerminator terminator;
} VTermStringFragment;

typedef union VTermValue {
  int boolean;
  int number;
  VTermStringFragment string;
  VTermColor color;
} VTermValue;

typedef struct VTermGlyphInfo {
  ScreenChar schar;
  int width;
  unsigned int protected_cell : 1;
  unsigned int dwl : 1;
  unsigned int dhl : 2;
} VTermGlyphInfo;

typedef struct VTermLineInfo {
  unsigned int doublewidth : 1;
  unsigned int doubleheight : 2;
  unsigned int continuation : 1;
} VTermLineInfo;

typedef struct VTermScreenCellAttrs {
  unsigned int bold : 1;
  unsigned int underline : 2;
  unsigned int italic : 1;
  unsigned int blink : 1;
  unsigned int reverse : 1;
  unsigned int conceal : 1;
  unsigned int strike : 1;
  unsigned int font : 4;
  unsigned int dwl : 1;
  unsigned int dhl : 2;
  unsigned int small : 1;
  unsigned int baseline : 2;
  unsigned int dim : 1;
  unsigned int overline : 1;
} VTermScreenCellAttrs;

typedef struct VTermScreenCell {
  ScreenChar schar;
  char width;
  VTermScreenCellAttrs attrs;
  VTermColor fg;
  VTermColor bg;
  int uri;
} VTermScreenCell;

typedef struct VTermPen {
  VTermColor fg;
  VTermColor bg;
  int uri;
  unsigned int bold : 1;
  unsigned int underline : 2;
  unsigned int italic : 1;
  unsigned int blink : 1;
  unsigned int reverse : 1;
  unsigned int conceal : 1;
  unsigned int strike : 1;
  unsigned int font : 4;
  unsigned int small : 1;
  unsigned int baseline : 2;
  unsigned int dim : 1;
  unsigned int overline : 1;
} VTermPen;

typedef struct ScreenPen {
  VTermColor fg;
  VTermColor bg;
  int uri;
  unsigned int bold : 1;
  unsigned int underline : 2;
  unsigned int italic : 1;
  unsigned int blink : 1;
  unsigned int reverse : 1;
  unsigned int conceal : 1;
  unsigned int strike : 1;
  unsigned int font : 4;
  unsigned int small : 1;
  unsigned int baseline : 2;
  unsigned int dim : 1;
  unsigned int overline : 1;
  unsigned int protected_cell : 1;
  unsigned int dwl : 1;
  unsigned int dhl : 2;
} ScreenPen;

typedef struct ScreenCell { ScreenChar schar; ScreenPen pen; } ScreenCell;

typedef struct VTermEncodingInstance { VTermEncoding *enc; char data[16]; } VTermEncodingInstance;

typedef struct VTermKeyEncodingFlags {
  bool disambiguate : 1;
  bool report_events : 1;
  bool report_alternate : 1;
  bool report_all_keys : 1;
  bool report_associated : 1;
} VTermKeyEncodingFlags;

typedef struct VTermKeyEncodingStack { VTermKeyEncodingFlags items[16]; uint8_t size; } VTermKeyEncodingStack;

typedef struct VTermState_mode {
  unsigned int keypad : 1;
  unsigned int cursor : 1;
  unsigned int autowrap : 1;
  unsigned int insert : 1;
  unsigned int newline : 1;
  unsigned int cursor_visible : 1;
  unsigned int cursor_blink : 1;
  unsigned int cursor_shape : 2;
  unsigned int alt_screen : 1;
  unsigned int origin : 1;
  unsigned int screen : 1;
  unsigned int leftrightmargin : 1;
  unsigned int bracketpaste : 1;
  unsigned int report_focus : 1;
  unsigned int theme_updates : 1;
  unsigned int synchronized_output : 1;
} VTermState_mode;

typedef struct VTermState_saved_mode {
  unsigned int cursor_visible : 1;
  unsigned int cursor_blink : 1;
  unsigned int cursor_shape : 2;
} VTermState_saved_mode;

typedef struct VTermState_saved { VTermPos pos; VTermPen pen; VTermState_saved_mode mode; } VTermState_saved;

typedef struct VTermState_tmp_selection {
  uint16_t mask;
  unsigned int state : 8;
  uint32_t recvpartial;
  uint32_t sendpartial;
} VTermState_tmp_selection;

typedef union VTermState_tmp { char decrqss[4]; VTermState_tmp_selection selection; } VTermState_tmp;

typedef struct VTermSelectionCallbacks VTermSelectionCallbacks;

typedef struct VTermState_selection {
  const VTermSelectionCallbacks *callbacks;
  void *user;
  char *buffer;
  size_t buflen;
} VTermState_selection;

typedef struct VTermStateCallbacks VTermStateCallbacks;
typedef struct VTermStateFallbacks VTermStateFallbacks;
typedef struct VTermParserCallbacks VTermParserCallbacks;
typedef struct VTermScreenCallbacks VTermScreenCallbacks;

struct VTermState {
  VTerm *vt;
  const VTermStateCallbacks *callbacks;
  void *cbdata;
  const VTermStateFallbacks *fallbacks;
  void *fbdata;
  int rows;
  int cols;
  VTermPos pos;
  int at_phantom;
  int scrollregion_top;
  int scrollregion_bottom;
  int scrollregion_left;
  int scrollregion_right;
  uint8_t *tabstops;
  VTermLineInfo *lineinfos[2];
  VTermLineInfo *lineinfo;
  int mouse_col;
  int mouse_row;
  int mouse_buttons;
  int mouse_flags;
  VTermState_mouse_protocol mouse_protocol;
  char grapheme_buf[32];
  size_t grapheme_len;
  uint32_t grapheme_last;
  GraphemeState grapheme_state;
  int combine_width;
  VTermPos combine_pos;
  VTermState_mode mode;
  VTermEncodingInstance encoding[4];
  VTermEncodingInstance encoding_utf8;
  int gl_set;
  int gr_set;
  int gsingle_set;
  VTermPen pen;
  VTermColor default_fg;
  VTermColor default_bg;
  VTermColor colors[16];
  int bold_is_highbright;
  unsigned int protected_cell : 1;
  VTermState_saved saved;
  VTermState_tmp tmp;
  VTermState_selection selection;
  VTermKeyEncodingStack key_encoding_stacks[2];
};

struct VTermScreen {
  VTerm *vt;
  VTermState *state;
  const VTermScreenCallbacks *callbacks;
  void *cbdata;
  VTermDamageSize damage_merge;
  VTermRect damaged;
  VTermRect pending_scrollrect;
  int pending_scroll_downward;
  int pending_scroll_rightward;
  int rows;
  int cols;
  unsigned int global_reverse : 1;
  unsigned int reflow : 1;
  ScreenCell *buffers[2];
  ScreenCell *buffer;
  VTermScreenCell *sb_buffer;
  ScreenPen pen;
};

typedef struct VTerm_mode { unsigned int utf8 : 1; unsigned int ctrl8bit : 1; } VTerm_mode;
typedef struct VTerm_parser_v_csi { int leaderlen; char leader[16]; int argi; long args[32]; } VTerm_parser_v_csi;
typedef struct VTerm_parser_v_osc { int command; } VTerm_parser_v_osc;
typedef struct VTerm_parser_v_dcs { int commandlen; char command[16]; } VTerm_parser_v_dcs;
typedef union VTerm_parser_v { VTerm_parser_v_csi csi; VTerm_parser_v_osc osc; VTerm_parser_v_dcs dcs; } VTerm_parser_v;
typedef struct VTerm_parser {
  VTermParserState state;
  bool in_esc : 1;
  int intermedlen;
  char intermed[16];
  VTerm_parser_v v;
  const VTermParserCallbacks *callbacks;
  void *cbdata;
  bool string_initial;
  bool emit_nul;
} VTerm_parser;
typedef void (*VTermOutputCallback)(const char *s, size_t len, void *user);

struct VTerm {
  int rows;
  int cols;
  VTerm_mode mode;
  VTerm_parser parser;
  VTermOutputCallback outfunc;
  void *outdata;
  char *outbuffer;
  size_t outbuffer_len;
  size_t outbuffer_cur;
  char *tmpbuffer;
  size_t tmpbuffer_len;
  VTermState *state;
  VTermScreen *screen;
};

typedef struct VTermStateFields { VTermPos pos; VTermLineInfo *lineinfos[2]; } VTermStateFields;

/* Callback tables.  Every slot that would take a struct BY VALUE is
 * declared with its System V x86-64 register decomposition instead --
 * see the header.  The layout (one function pointer per slot) is
 * unchanged, which is what v0's sizeof rows assert. */
struct VTermParserCallbacks {
  int (*text)(const char *bytes, size_t len, void *user);
  int (*control)(uint8_t control, void *user);
  int (*escape)(const char *bytes, size_t len, void *user);
  int (*csi)(const char *leader, const long args[], int argcount, const char *intermed, char command, void *user);
  int (*osc)(int command, uint64_t f0, uint64_t f1, void *user);
  int (*dcs)(const char *command, size_t commandlen, uint64_t f0, uint64_t f1, void *user);
  int (*apc)(uint64_t f0, uint64_t f1, void *user);
  int (*pm)(uint64_t f0, uint64_t f1, void *user);
  int (*sos)(uint64_t f0, uint64_t f1, void *user);
  int (*resize)(int rows, int cols, void *user);
};

struct VTermStateCallbacks {
  int (*putglyph)(VTermGlyphInfo *info, uint64_t pos, void *user);
  int (*movecursor)(uint64_t pos, uint64_t oldpos, int visible, void *user);
  int (*scrollrect)(uint64_t r0, uint64_t r1, int downward, int rightward, void *user);
  int (*moverect)(uint64_t d0, uint64_t d1, uint64_t s0, uint64_t s1, void *user);
  int (*erase)(uint64_t r0, uint64_t r1, int selective, void *user);
  int (*initpen)(void *user);
  int (*setpenattr)(VTermAttr attr, VTermValue *val, void *user);
  int (*settermprop)(VTermProp prop, VTermValue *val, void *user);
  int (*bell)(void *user);
  int (*resize)(int rows, int cols, VTermStateFields *fields, void *user);
  int (*theme)(bool *is_dark, void *user);
  int (*setlineinfo)(int row, const VTermLineInfo *newinfo, const VTermLineInfo *oldinfo, void *user);
  int (*sb_clear)(void *user);
};

struct VTermStateFallbacks {
  int (*control)(uint8_t control, void *user);
  int (*csi)(const char *leader, const long args[], int argcount, const char *intermed, char command, void *user);
  int (*osc)(int command, uint64_t f0, uint64_t f1, void *user);
  int (*dcs)(const char *command, size_t commandlen, uint64_t f0, uint64_t f1, void *user);
  int (*apc)(uint64_t f0, uint64_t f1, void *user);
  int (*pm)(uint64_t f0, uint64_t f1, void *user);
  int (*sos)(uint64_t f0, uint64_t f1, void *user);
};

struct VTermScreenCallbacks {
  int (*damage)(uint64_t r0, uint64_t r1, void *user);
  int (*moverect)(uint64_t d0, uint64_t d1, uint64_t s0, uint64_t s1, void *user);
  int (*movecursor)(uint64_t pos, uint64_t oldpos, int visible, void *user);
  int (*settermprop)(VTermProp prop, VTermValue *val, void *user);
  int (*bell)(void *user);
  int (*resize)(int rows, int cols, void *user);
  int (*theme)(bool *is_dark, void *user);
  int (*sb_pushline)(int cols, const VTermScreenCell *cells, void *user);
  int (*sb_popline)(int cols, VTermScreenCell *cells, void *user);
  int (*sb_clear)(void *user);
};

struct VTermSelectionCallbacks {
  int (*set)(VTermSelectionMask mask, uint64_t f0, uint64_t f1, void *user);
  int (*query)(VTermSelectionMask mask, void *user);
};

/* The thirty exported entry points this sweep drives. */
VTerm *vterm_new(int rows, int cols);
void vterm_set_size(VTerm *vt, int rows, int cols);
void vterm_set_utf8(VTerm *vt, int is_utf8);
size_t vterm_input_write(VTerm *vt, const char *bytes, size_t len);
void vterm_parser_set_callbacks(VTerm *vt, const VTermParserCallbacks *cbs, void *user);
void vterm_output_set_callback(VTerm *vt, VTermOutputCallback func, void *user);
VTermState *vterm_obtain_state(VTerm *vt);
VTermScreen *vterm_obtain_screen(VTerm *vt);
void vterm_state_reset(VTermState *state, int hard);
void vterm_state_set_callbacks(VTermState *state, const VTermStateCallbacks *cbs, void *user);
void vterm_state_set_unrecognised_fallbacks(VTermState *state, const VTermStateFallbacks *fbs, void *user);
void vterm_state_set_selection_callbacks(VTermState *state, const VTermSelectionCallbacks *cbs, void *user, char *buffer, size_t buflen);
void vterm_state_focus_in(VTermState *state);
void vterm_state_focus_out(VTermState *state);
const VTermLineInfo *vterm_state_get_lineinfo(const VTermState *state, int row);
void vterm_screen_reset(VTermScreen *screen, int hard);
int vterm_screen_get_cell(const VTermScreen *screen, VTermPos pos, VTermScreenCell *cell);
void vterm_screen_set_callbacks(VTermScreen *screen, const VTermScreenCallbacks *cbs, void *user);
void vterm_screen_enable_altscreen(VTermScreen *screen, int altscreen);
void vterm_screen_enable_reflow(VTermScreen *screen, bool reflow);
void vterm_screen_convert_color_to_rgb(const VTermScreen *screen, VTermColor *col);
ScreenCell *getcell(const VTermScreen *screen, int row, int col);
void vterm_keyboard_unichar(VTerm *vt, uint32_t c, VTermModifier mod);
void vterm_keyboard_key(VTerm *vt, VTermKey key, VTermModifier mod);
void vterm_keyboard_start_paste(VTerm *vt);
void vterm_keyboard_end_paste(VTerm *vt);
void vterm_mouse_move(VTerm *vt, int row, int col, VTermModifier mod);
void vterm_mouse_button(VTerm *vt, int button, bool pressed, VTermModifier mod);
VTermEncoding *vterm_lookup_encoding(VTermEncodingType type, char designation);
size_t schar_get(char *buf_out, ScreenChar sc);
]])

local C = ffi.C

-- ------------------------------------------------------------- plumbing

local only = os.getenv('VTSWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('VTSWEEP_TRACE') == '1'
local probe = os.getenv('VT_PROBE')
if probe == '' then
  probe = nil
end

io.stdout:setvbuf('line')

local rows_emitted = 0
local function emit(...)
  rows_emitted = rows_emitted + 1
  io.write(table.concat({ ... }, ' '), '\n')
end

local function esc(s)
  return (tostring(s):gsub('[%z\1-\31\92\127-\255]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

--- FNV-1a over a string, so a capped answer still differs when its tail
--- moves.  Deterministic, 32-bit, no library dependency beyond `bit`.
local function fnv(s)
  local h = 2166136261
  for i = 1, #s do
    h = bit.bxor(h, s:byte(i))
    h = bit.band(h * 16777619, 0xffffffff)
  end
  return bit.band(h, 0xffffffff)
end

local function cap(s, limit)
  limit = limit or 1400
  s = tostring(s)
  if #s <= limit then
    return s
  end
  return string.format('%s...<+%d,h=%08x>', s:sub(1, limit), #s - limit, fnv(s))
end

local SEEN = {}
local function label_once(label)
  if SEEN[label] then
    emit('!!', 'DUPLICATE', 'LABEL', label)
  end
  SEEN[label] = true
  return label
end

-- -------------------------------------------------------------- struct

-- A v91 child inherits $VT_STRUCT.  It must NOT open it: `io.open(_, 'w')`
-- truncates, and the last child to run would leave the parent's canonical
-- JSON one line long while the report looked perfectly healthy.
local structpath = (not probe) and os.getenv('VT_STRUCT') or nil
local structfd = structpath and io.open(structpath, 'w') or nil

local function q(str)
  return '"'
    .. (tostring(str):gsub('[%c"\\\128-\255]', function(c)
      return string.format('\\x%02x', c:byte())
    end))
    .. '"'
end

local canon
function canon(value)
  local t = type(value)
  if t == 'string' then
    return q(value)
  elseif t == 'number' then
    if value == math.floor(value) and math.abs(value) < 2 ^ 53 then
      return string.format('%d', value)
    end
    return string.format('%.17g', value)
  elseif t ~= 'table' then
    return q(tostring(value))
  end
  local n = 0
  for _ in pairs(value) do
    n = n + 1
  end
  if n == #value then
    local parts = {}
    for i, item in ipairs(value) do
      parts[i] = canon(item)
    end
    return '[' .. table.concat(parts, ',') .. ']'
  end
  local keys = {}
  for k in pairs(value) do
    keys[#keys + 1] = tostring(k)
  end
  table.sort(keys)
  local parts = {}
  for _, k in ipairs(keys) do
    parts[#parts + 1] = q(k) .. ':' .. canon(value[k])
  end
  return '{' .. table.concat(parts, ',') .. '}'
end

local function struct(label, value)
  if structfd then
    structfd:write(label, '\t', canon(value), '\n')
  end
end

-- ------------------------------------------------------------ constants

local MOD = { NONE = 0, SHIFT = 1, ALT = 2, CTRL = 4 }

local KEY = {
  NONE = 0,
  ENTER = 1,
  TAB = 2,
  BACKSPACE = 3,
  ESCAPE = 4,
  UP = 5,
  DOWN = 6,
  LEFT = 7,
  RIGHT = 8,
  INS = 9,
  DEL = 10,
  HOME = 11,
  END = 12,
  PAGEUP = 13,
  PAGEDOWN = 14,
  FUNCTION_0 = 256,
  KP_0 = 512,
}

local PROPNAME = {
  [1] = 'CURSORVISIBLE',
  [2] = 'CURSORBLINK',
  [3] = 'ALTSCREEN',
  [4] = 'TITLE',
  [5] = 'ICONNAME',
  [6] = 'REVERSE',
  [7] = 'CURSORSHAPE',
  [8] = 'MOUSE',
  [9] = 'FOCUSREPORT',
  [10] = 'THEMEUPDATES',
  [11] = 'SYNCOUTPUT',
}
-- 1 bool, 2 int, 3 string, 4 color
local PROPTYPE = {
  [1] = 1,
  [2] = 1,
  [3] = 1,
  [4] = 3,
  [5] = 3,
  [6] = 1,
  [7] = 2,
  [8] = 2,
  [9] = 1,
  [10] = 1,
  [11] = 1,
}

local ATTRNAME = {
  [1] = 'BOLD',
  [2] = 'UNDERLINE',
  [3] = 'ITALIC',
  [4] = 'BLINK',
  [5] = 'REVERSE',
  [6] = 'CONCEAL',
  [7] = 'STRIKE',
  [8] = 'FONT',
  [9] = 'FOREGROUND',
  [10] = 'BACKGROUND',
  [11] = 'SMALL',
  [12] = 'BASELINE',
  [13] = 'URI',
  [14] = 'DIM',
  [15] = 'OVERLINE',
}
local ATTRTYPE = {
  [1] = 1,
  [2] = 2,
  [3] = 1,
  [4] = 1,
  [5] = 1,
  [6] = 1,
  [7] = 1,
  [8] = 2,
  [9] = 4,
  [10] = 4,
  [11] = 1,
  [12] = 2,
  [13] = 2,
  [14] = 1,
  [15] = 1,
}

local ESC = '\27'
local ST = ESC .. '\\'
local BEL = '\7'
local CSI = ESC .. '['

-- ------------------------------------------------------- scratch decode

local W2 = ffi.new('uint64_t[2]')
local W2RECT = ffi.cast('VTermRect *', W2)
local W2POS = ffi.cast('VTermPos *', W2)
local W2FRAG = ffi.cast('VTermStringFragment *', W2)
local CELL = ffi.new('VTermScreenCell')
local POS = ffi.new('VTermPos')
local SBUF = ffi.new('char[64]')

local function rect2(a, b)
  W2[0] = a
  W2[1] = b
  return string.format(
    '%d,%d,%d,%d',
    W2RECT.start_row,
    W2RECT.end_row,
    W2RECT.start_col,
    W2RECT.end_col
  )
end

local function pos1(a)
  W2[0] = a
  return string.format('%d,%d', W2POS.row, W2POS.col)
end

local function frag2(a, b)
  W2[0] = a
  W2[1] = b
  local len = tonumber(W2FRAG.len)
  local text = (W2FRAG.str ~= nil and len > 0) and ffi.string(W2FRAG.str, len) or ''
  return string.format(
    'i%df%dt%d:%s',
    W2FRAG.initial and 1 or 0,
    W2FRAG.final and 1 or 0,
    tonumber(W2FRAG.terminator),
    esc(text)
  )
end

local function scharstr(sc)
  if sc == 0 then
    return ''
  end
  local n = tonumber(C.schar_get(SBUF, sc))
  return ffi.string(SBUF, n)
end

local function colorsig(col)
  local ty = col.type
  local base
  if bit.band(ty, 1) == 1 then
    base = 'i' .. col.indexed.idx
  else
    base = string.format('r%02x%02x%02x', col.rgb.red, col.rgb.green, col.rgb.blue)
  end
  local d = bit.band(ty, 6)
  if d ~= 0 then
    base = 'd' .. d .. base
  end
  return base
end

local function attrbits(a)
  return a.bold
    + a.underline * 2
    + a.italic * 8
    + a.blink * 16
    + a.reverse * 32
    + a.conceal * 64
    + a.strike * 128
    + a.font * 256
    + a.dwl * 4096
    + a.dhl * 8192
    + a.small * 32768
    + a.baseline * 65536
    + a.dim * 262144
    + a.overline * 524288
end

local function pensig(p)
  return string.format(
    'fg=%s bg=%s uri=%d b%d u%d i%d k%d r%d c%d s%d f%d sm%d bl%d d%d o%d',
    colorsig(p.fg),
    colorsig(p.bg),
    p.uri,
    p.bold,
    p.underline,
    p.italic,
    p.blink,
    p.reverse,
    p.conceal,
    p.strike,
    p.font,
    p.small,
    p.baseline,
    p.dim,
    p.overline
  )
end

local function modesig(m)
  return string.format(
    'kp%d cu%d aw%d in%d nl%d cv%d cb%d cs%d as%d or%d sc%d lr%d bp%d rf%d tu%d so%d',
    m.keypad,
    m.cursor,
    m.autowrap,
    m.insert,
    m.newline,
    m.cursor_visible,
    m.cursor_blink,
    m.cursor_shape,
    m.alt_screen,
    m.origin,
    m.screen,
    m.leftrightmargin,
    m.bracketpaste,
    m.report_focus,
    m.theme_updates,
    m.synchronized_output
  )
end

local function kesig(st)
  local out = {}
  for i = 0, 1 do
    local stack = st.key_encoding_stacks[i]
    local items = {}
    for j = 0, tonumber(stack.size) - 1 do
      local f = stack.items[j]
      items[#items + 1] = string.format(
        '%d%d%d%d%d',
        f.disambiguate and 1 or 0,
        f.report_events and 1 or 0,
        f.report_alternate and 1 or 0,
        f.report_all_keys and 1 or 0,
        f.report_associated and 1 or 0
      )
    end
    out[#out + 1] = string.format('s%d=%d[%s]', i, tonumber(stack.size), table.concat(items, ','))
  end
  return table.concat(out, ' ')
end

local function statesig(st)
  if st == nil then
    return 'nostate'
  end
  local colors = {}
  for i = 0, 15 do
    colors[#colors + 1] = colorsig(st.colors[i])
  end
  return string.format(
    'pos=%d,%d ph=%d sr=%d,%d,%d,%d mouse=%d,%d,%d,%d,%d gl=%d glast=%d gst=%d cw=%d cpos=%d,%d set=%d,%d,%d bh=%d prot=%d mode{%s} pen{%s} dfg=%s dbg=%s pal=[%s] saved{%d,%d %s cv%d cb%d cs%d} sel{m%d s%d r%d w%d buflen%d} kes{%s} size=%dx%d',
    st.pos.row,
    st.pos.col,
    st.at_phantom,
    st.scrollregion_top,
    st.scrollregion_bottom,
    st.scrollregion_left,
    st.scrollregion_right,
    st.mouse_row,
    st.mouse_col,
    st.mouse_buttons,
    st.mouse_flags,
    tonumber(st.mouse_protocol),
    tonumber(st.grapheme_len),
    tonumber(st.grapheme_last),
    tonumber(st.grapheme_state),
    st.combine_width,
    st.combine_pos.row,
    st.combine_pos.col,
    st.gl_set,
    st.gr_set,
    st.gsingle_set,
    st.bold_is_highbright,
    st.protected_cell,
    modesig(st.mode),
    pensig(st.pen),
    colorsig(st.default_fg),
    colorsig(st.default_bg),
    table.concat(colors, ','),
    st.saved.pos.row,
    st.saved.pos.col,
    pensig(st.saved.pen),
    st.saved.mode.cursor_visible,
    st.saved.mode.cursor_blink,
    st.saved.mode.cursor_shape,
    tonumber(st.tmp.selection.mask),
    tonumber(st.tmp.selection.state),
    tonumber(st.tmp.selection.recvpartial),
    tonumber(st.tmp.selection.sendpartial),
    tonumber(st.selection.buflen),
    kesig(st),
    st.rows,
    st.cols
  )
end

local function valsig(kind, val)
  if kind == 1 then
    return 'b' .. val.boolean
  elseif kind == 2 then
    return 'n' .. val.number
  elseif kind == 3 then
    W2[0] = 0
    W2[1] = 0
    local f = val.string
    local len = tonumber(f.len)
    local text = (f.str ~= nil and len > 0) and ffi.string(f.str, len) or ''
    return string.format(
      's i%df%dt%d:%s',
      f.initial and 1 or 0,
      f.final and 1 or 0,
      tonumber(f.terminator),
      esc(text)
    )
  elseif kind == 4 then
    return 'c' .. colorsig(val.color)
  end
  return '?'
end

-- ------------------------------------------------------------ callbacks

--- Everything a case observed.  One live recorder at a time; the sweep is
--- single-threaded and every callback fires inside an `input_write` or a
--- `reset` we called ourselves.
local REC = { ev = {}, out = {} }

local function ev(...)
  local n = #REC.ev
  if n < 400 then
    REC.ev[n + 1] = table.concat({ ... }, ' ')
  elseif n == 400 then
    REC.ev[401] = '<truncated>'
  end
end

local ANCHOR = {}
local function cb(sig, fn)
  local c = ffi.cast(sig, fn)
  ANCHOR[#ANCHOR + 1] = c
  return c
end

local OUTCB = cb('void (*)(const char *, size_t, void *)', function(s, len, _)
  REC.out[#REC.out + 1] = ffi.string(s, tonumber(len))
end)

local PARSER_CBS = ffi.new('VTermParserCallbacks')
PARSER_CBS.text = cb('int (*)(const char *, size_t, void *)', function(b, len, _)
  ev('text', esc(ffi.string(b, tonumber(len))))
  return tonumber(len)
end)
PARSER_CBS.control = cb('int (*)(uint8_t, void *)', function(c, _)
  ev(string.format('control %02x', c))
  return 1
end)
PARSER_CBS.escape = cb('int (*)(const char *, size_t, void *)', function(b, len, _)
  ev('escape', esc(ffi.string(b, tonumber(len))))
  return 1
end)

local function csisig(leader, args, argcount, intermed, command)
  local parts = {}
  for i = 0, argcount - 1 do
    parts[#parts + 1] = string.format('%d', tonumber(args[i]))
  end
  return string.format(
    'l=%s a=[%s] i=%s c=%s',
    leader ~= nil and esc(ffi.string(leader)) or '-',
    table.concat(parts, ','),
    intermed ~= nil and esc(ffi.string(intermed)) or '-',
    esc(string.char(command % 256))
  )
end

PARSER_CBS.csi = cb(
  'int (*)(const char *, const long *, int, const char *, char, void *)',
  function(leader, args, argcount, intermed, command, _)
    ev('csi', csisig(leader, args, argcount, intermed, command))
    return 1
  end
)
PARSER_CBS.osc = cb('int (*)(int, uint64_t, uint64_t, void *)', function(command, a, b, _)
  ev('osc', command, frag2(a, b))
  return 1
end)
PARSER_CBS.dcs = cb(
  'int (*)(const char *, size_t, uint64_t, uint64_t, void *)',
  function(command, len, a, b, _)
    ev('dcs', esc(ffi.string(command, tonumber(len))), frag2(a, b))
    return 1
  end
)
PARSER_CBS.apc = cb('int (*)(uint64_t, uint64_t, void *)', function(a, b, _)
  ev('apc', frag2(a, b))
  return 1
end)
PARSER_CBS.pm = cb('int (*)(uint64_t, uint64_t, void *)', function(a, b, _)
  ev('pm', frag2(a, b))
  return 1
end)
PARSER_CBS.sos = cb('int (*)(uint64_t, uint64_t, void *)', function(a, b, _)
  ev('sos', frag2(a, b))
  return 1
end)
PARSER_CBS.resize = cb('int (*)(int, int, void *)', function(r, c, _)
  ev('presize', r, c)
  return 1
end)

local STATE_CBS = ffi.new('VTermStateCallbacks')
STATE_CBS.putglyph = cb('int (*)(VTermGlyphInfo *, uint64_t, void *)', function(info, p, _)
  ev(
    'putglyph',
    esc(scharstr(info.schar)),
    string.format('w%d p%d dwl%d dhl%d @%s', info.width, info.protected_cell, info.dwl, info.dhl, pos1(p))
  )
  return 1
end)
STATE_CBS.movecursor = cb('int (*)(uint64_t, uint64_t, int, void *)', function(p, o, vis, _)
  local np = pos1(p)
  ev('movecursor', np, '<-', pos1(o), 'vis=' .. vis)
  return 1
end)
STATE_CBS.scrollrect = cb('int (*)(uint64_t, uint64_t, int, int, void *)', function(a, b, dn, rt, _)
  ev('scrollrect', rect2(a, b), 'd=' .. dn, 'r=' .. rt)
  return 0
end)
STATE_CBS.moverect = cb(
  'int (*)(uint64_t, uint64_t, uint64_t, uint64_t, void *)',
  function(d0, d1, s0, s1, _)
    local d = rect2(d0, d1)
    ev('moverect', d, '<-', rect2(s0, s1))
    return 0
  end
)
STATE_CBS.erase = cb('int (*)(uint64_t, uint64_t, int, void *)', function(a, b, sel, _)
  ev('erase', rect2(a, b), 'sel=' .. sel)
  return 1
end)
STATE_CBS.initpen = cb('int (*)(void *)', function(_)
  ev('initpen')
  return 1
end)
STATE_CBS.setpenattr = cb('int (*)(VTermAttr, VTermValue *, void *)', function(attr, val, _)
  ev('setpenattr', ATTRNAME[tonumber(attr)] or ('?' .. tonumber(attr)), valsig(ATTRTYPE[tonumber(attr)] or 2, val))
  return 1
end)
STATE_CBS.settermprop = cb('int (*)(VTermProp, VTermValue *, void *)', function(prop, val, _)
  ev('settermprop', PROPNAME[tonumber(prop)] or ('?' .. tonumber(prop)), valsig(PROPTYPE[tonumber(prop)] or 2, val))
  return 1
end)
STATE_CBS.bell = cb('int (*)(void *)', function(_)
  ev('bell')
  return 1
end)
STATE_CBS.resize = cb('int (*)(int, int, VTermStateFields *, void *)', function(r, c, f, _)
  ev('sresize', r, c, string.format('pos=%d,%d', f.pos.row, f.pos.col))
  return 1
end)
STATE_CBS.theme = cb('int (*)(bool *, void *)', function(dark, _)
  dark[0] = true
  ev('theme')
  return 1
end)
STATE_CBS.setlineinfo = cb(
  'int (*)(int, const VTermLineInfo *, const VTermLineInfo *, void *)',
  function(row, newinfo, oldinfo, _)
    ev(
      'setlineinfo',
      row,
      string.format('dw%d dh%d co%d', newinfo.doublewidth, newinfo.doubleheight, newinfo.continuation),
      oldinfo ~= nil
          and string.format('<- dw%d dh%d co%d', oldinfo.doublewidth, oldinfo.doubleheight, oldinfo.continuation)
        or '<- -'
    )
    return 1
  end
)
STATE_CBS.sb_clear = cb('int (*)(void *)', function(_)
  ev('state_sb_clear')
  return 1
end)

local FALLBACK_CBS = ffi.new('VTermStateFallbacks')
FALLBACK_CBS.control = cb('int (*)(uint8_t, void *)', function(c, _)
  ev(string.format('fb_control %02x', c))
  return 1
end)
FALLBACK_CBS.csi = cb(
  'int (*)(const char *, const long *, int, const char *, char, void *)',
  function(leader, args, argcount, intermed, command, _)
    ev('fb_csi', csisig(leader, args, argcount, intermed, command))
    return 1
  end
)
FALLBACK_CBS.osc = cb('int (*)(int, uint64_t, uint64_t, void *)', function(command, a, b, _)
  ev('fb_osc', command, frag2(a, b))
  return 1
end)
FALLBACK_CBS.dcs = cb(
  'int (*)(const char *, size_t, uint64_t, uint64_t, void *)',
  function(command, len, a, b, _)
    ev('fb_dcs', esc(ffi.string(command, tonumber(len))), frag2(a, b))
    return 1
  end
)
FALLBACK_CBS.apc = cb('int (*)(uint64_t, uint64_t, void *)', function(a, b, _)
  ev('fb_apc', frag2(a, b))
  return 1
end)
FALLBACK_CBS.pm = cb('int (*)(uint64_t, uint64_t, void *)', function(a, b, _)
  ev('fb_pm', frag2(a, b))
  return 1
end)
FALLBACK_CBS.sos = cb('int (*)(uint64_t, uint64_t, void *)', function(a, b, _)
  ev('fb_sos', frag2(a, b))
  return 1
end)

local SEL_CBS = ffi.new('VTermSelectionCallbacks')
SEL_CBS.set = cb('int (*)(VTermSelectionMask, uint64_t, uint64_t, void *)', function(mask, a, b, _)
  ev('sel_set', tonumber(mask), frag2(a, b))
  return 1
end)
SEL_CBS.query = cb('int (*)(VTermSelectionMask, void *)', function(mask, _)
  ev('sel_query', tonumber(mask))
  return 1
end)

local SCREEN_CBS = ffi.new('VTermScreenCallbacks')
SCREEN_CBS.damage = cb('int (*)(uint64_t, uint64_t, void *)', function(a, b, _)
  ev('damage', rect2(a, b))
  return 1
end)
SCREEN_CBS.moverect = cb(
  'int (*)(uint64_t, uint64_t, uint64_t, uint64_t, void *)',
  function(d0, d1, s0, s1, _)
    ev('c_moverect', rect2(d0, d1), '<-', rect2(s0, s1))
    return REC.want_moverect and 1 or 0
  end
)
SCREEN_CBS.movecursor = cb('int (*)(uint64_t, uint64_t, int, void *)', function(p, o, vis, _)
  ev('c_movecursor', pos1(p), '<-', pos1(o), 'vis=' .. vis)
  return 1
end)
SCREEN_CBS.settermprop = cb('int (*)(VTermProp, VTermValue *, void *)', function(prop, val, _)
  ev('c_settermprop', PROPNAME[tonumber(prop)] or ('?' .. tonumber(prop)), valsig(PROPTYPE[tonumber(prop)] or 2, val))
  return 1
end)
SCREEN_CBS.bell = cb('int (*)(void *)', function(_)
  ev('c_bell')
  return 1
end)
SCREEN_CBS.resize = cb('int (*)(int, int, void *)', function(r, c, _)
  ev('c_resize', r, c)
  return 1
end)
SCREEN_CBS.theme = cb('int (*)(bool *, void *)', function(dark, _)
  dark[0] = true
  ev('c_theme')
  return 1
end)
SCREEN_CBS.sb_pushline = cb(
  'int (*)(int, const VTermScreenCell *, void *)',
  function(cols, cells, _)
    local text = {}
    for i = 0, cols - 1 do
      text[#text + 1] = scharstr(cells[i].schar)
    end
    local line = table.concat(text)
    REC.sb[#REC.sb + 1] = line
    ev('sb_pushline', cols, esc(line))
    return 1
  end
)
SCREEN_CBS.sb_popline = cb('int (*)(int, VTermScreenCell *, void *)', function(cols, cells, _)
  local line = table.remove(REC.sb)
  if line == nil then
    ev('sb_popline', cols, 'EMPTY')
    return 0
  end
  for i = 0, cols - 1 do
    local byte = line:byte(i + 1)
    cells[i].schar = byte or 0
    cells[i].width = 1
  end
  ev('sb_popline', cols, esc(line))
  return 1
end)
SCREEN_CBS.sb_clear = cb('int (*)(void *)', function(_)
  REC.sb = {}
  ev('sb_clear')
  return 1
end)

-- ---------------------------------------------------------- the terminal

local Term = {}
Term.__index = Term

--- A fresh emulator.  `vterm_free` is not exported, so nothing is ever
--- released; keep the geometry small.
local function newterm(o)
  o = o or {}
  local rows, cols = o.rows or 6, o.cols or 20
  REC = { ev = {}, out = {}, sb = {}, want_moverect = o.want_moverect }
  local vt = C.vterm_new(rows, cols)
  C.vterm_set_utf8(vt, o.utf8 == false and 0 or 1)
  C.vterm_output_set_callback(vt, OUTCB, nil)
  local t = setmetatable({ vt = vt, rows = rows, cols = cols, mode = o.mode or 'screen' }, Term)
  if t.mode == 'parser' then
    C.vterm_parser_set_callbacks(vt, PARSER_CBS, nil)
  elseif t.mode == 'state' then
    t.state = C.vterm_obtain_state(vt)
    C.vterm_state_set_callbacks(t.state, STATE_CBS, nil)
    if o.fallbacks then
      C.vterm_state_set_unrecognised_fallbacks(t.state, FALLBACK_CBS, nil)
    end
    if o.selection then
      C.vterm_state_set_selection_callbacks(t.state, SEL_CBS, nil, nil, o.selbuflen or 128)
    end
    C.vterm_state_reset(t.state, 1)
  else
    t.screen = C.vterm_obtain_screen(vt)
    t.state = C.vterm_obtain_state(vt)
    if o.altscreen then
      C.vterm_screen_enable_altscreen(t.screen, 1)
    end
    if o.reflow ~= nil then
      C.vterm_screen_enable_reflow(t.screen, o.reflow and true or false)
    end
    C.vterm_screen_set_callbacks(t.screen, SCREEN_CBS, nil)
    if o.selection then
      C.vterm_state_set_selection_callbacks(t.state, SEL_CBS, nil, nil, o.selbuflen or 128)
    end
    C.vterm_screen_reset(t.screen, 1)
    if o.damage then
      -- `vterm_screen_set_damage_merge` is not an exported symbol.  Its
      -- body is a flush followed by this assignment, and the screen was
      -- just reset (nothing is pending), so the assignment alone is the
      -- whole operation.
      t.screen.damage_merge = o.damage
    end
  end
  t.rec = REC
  REC.ev = {}
  REC.out = {}
  return t
end

function Term:write(s)
  return tonumber(C.vterm_input_write(self.vt, s, #s))
end

--- Write `s` in pieces.  A parser that keeps no state across calls
--- answers the same thing for every split; one that does, does not.
function Term:writesplit(s, how)
  local n = 0
  if how == 'whole' or #s < 2 then
    n = self:write(s)
  elseif how == 'bytes' then
    for i = 1, #s do
      n = n + self:write(s:sub(i, i))
    end
  elseif how == 'half' then
    local mid = math.floor(#s / 2)
    n = self:write(s:sub(1, mid)) + self:write(s:sub(mid + 1))
  elseif how == 'edges' then
    n = self:write(s:sub(1, 1)) + self:write(s:sub(2, #s - 1)) + self:write(s:sub(#s))
  end
  return n
end

function Term:events()
  return table.concat(self.rec.ev, ' | ')
end

function Term:output()
  return esc(table.concat(self.rec.out))
end

function Term:reset()
  self.rec.ev = {}
  self.rec.out = {}
end

local function lineinfosig(li)
  if li == nil then
    return '?'
  end
  return string.format('%d%d%d:', li.doublewidth, li.doubleheight, li.continuation)
end

local function cellsig(cell)
  return string.format(
    '%s|%d|%x|%s|%s|%d',
    esc(scharstr(cell.schar)),
    cell.width,
    attrbits(cell.attrs),
    colorsig(cell.fg),
    colorsig(cell.bg),
    cell.uri
  )
end

--- The whole grid, run-length encoded per row, plus the row's lineinfo.
--- Complete: nothing is summarised away, and a mostly-blank row is two
--- runs.
function Term:grid()
  if self.screen == nil then
    return 'noscreen'
  end
  local out = {}
  for r = 0, self.rows - 1 do
    local runs, prev, n = {}, nil, 0
    for c = 0, self.cols - 1 do
      POS.row, POS.col = r, c
      local sig
      if C.vterm_screen_get_cell(self.screen, POS, CELL) == 0 then
        sig = 'OOB'
      else
        sig = cellsig(CELL)
      end
      if sig == prev then
        n = n + 1
      else
        if prev then
          runs[#runs + 1] = (n > 1 and (n .. '*') or '') .. prev
        end
        prev, n = sig, 1
      end
    end
    runs[#runs + 1] = (n > 1 and (n .. '*') or '') .. prev
    out[#out + 1] = string.format(
      'r%d[%s%s]',
      r,
      lineinfosig(C.vterm_state_get_lineinfo(self.state, r)),
      table.concat(runs, ',')
    )
  end
  return table.concat(out, ' ')
end

--- The screen's residual damage bookkeeping.  Under any merge level
--- above CELL the damage that has not been emitted yet lives here, and
--- `vterm_screen_flush_damage` is not exported -- so this IS the answer
--- for the merged part.
function Term:pending()
  if self.screen == nil then
    return '-'
  end
  local s = self.screen
  return string.format(
    'merge=%d damaged=%d,%d,%d,%d scrollrect=%d,%d,%d,%d dn=%d rt=%d rev=%d reflow=%d',
    tonumber(s.damage_merge),
    s.damaged.start_row,
    s.damaged.end_row,
    s.damaged.start_col,
    s.damaged.end_col,
    s.pending_scrollrect.start_row,
    s.pending_scrollrect.end_row,
    s.pending_scrollrect.start_col,
    s.pending_scrollrect.end_col,
    s.pending_scroll_downward,
    s.pending_scroll_rightward,
    s.global_reverse,
    s.reflow
  )
end

function Term:state_sig()
  return statesig(self.state)
end

function Term:parser_sig()
  local p = self.vt.parser
  return string.format(
    'pstate=%d esc=%d ilen=%d i=%s init=%d nul=%d',
    tonumber(p.state),
    p.in_esc and 1 or 0,
    p.intermedlen,
    esc(ffi.string(p.intermed, math.max(0, math.min(16, p.intermedlen)))),
    p.string_initial and 1 or 0,
    p.emit_nul and 1 or 0
  )
end

-- ------------------------------------------------------------- reporting

--- One row per case.  `parts` is a list of `key=value` strings; the row
--- is the label, the parts, and nothing else.
local function answer(label, parts, structv)
  label_once(label)
  emit(label, cap(table.concat(parts, ' ')))
  if structv then
    struct(label, structv)
  end
end

--- The standard answer for a screen-mode case.
local function screen_answer(label, t, extra)
  local parts = {}
  if extra then
    parts[#parts + 1] = extra
  end
  parts[#parts + 1] = 'ev={' .. cap(t:events(), 700) .. '}'
  parts[#parts + 1] = 'out=' .. t:output()
  parts[#parts + 1] = 'pend{' .. t:pending() .. '}'
  parts[#parts + 1] = 'st{' .. t:state_sig() .. '}'
  parts[#parts + 1] = 'grid{' .. cap(t:grid(), 900) .. '}'
  answer(label, parts, {
    ev = t.rec.ev,
    out = table.concat(t.rec.out),
    pending = t:pending(),
    state = t:state_sig(),
    grid = t:grid(),
  })
end

local function state_answer(label, t, extra)
  local parts = {}
  if extra then
    parts[#parts + 1] = extra
  end
  parts[#parts + 1] = 'ev={' .. cap(t:events(), 900) .. '}'
  parts[#parts + 1] = 'out=' .. t:output()
  parts[#parts + 1] = 'st{' .. t:state_sig() .. '}'
  answer(label, parts, { ev = t.rec.ev, out = table.concat(t.rec.out), state = t:state_sig() })
end

local function parser_answer(label, t, extra)
  local parts = {}
  if extra then
    parts[#parts + 1] = extra
  end
  parts[#parts + 1] = 'ev={' .. cap(t:events(), 900) .. '}'
  parts[#parts + 1] = 'out=' .. t:output()
  parts[#parts + 1] = 'p{' .. t:parser_sig() .. '}'
  answer(label, parts, { ev = t.rec.ev, out = table.concat(t.rec.out), parser = t:parser_sig() })
end

-- -------------------------------------------------------------- sections

local SECTIONS = {}
local function section(name, fn)
  SECTIONS[#SECTIONS + 1] = { name = name, fn = fn }
end

local function run_sections()
  for _, s in ipairs(SECTIONS) do
    if not only or s.name:match(only) then
      if trace then
        io.stderr:write('== ' .. s.name .. '\n')
      end
      local before = rows_emitted
      emit('##', s.name)
      local ok, err = pcall(s.fn)
      if not ok then
        emit('##', s.name, 'RAISED', esc(tostring(err)))
      end
      emit('##', s.name, string.format('rows=%d', rows_emitted - before - 1))
    end
  end
  emit('##', 'TOTAL', string.format('rows=%d', rows_emitted))
end

-- ============================================================== v0 canary

section('v0-canary', function()
  local TYPES = {
    'VTermPos',
    'VTermRect',
    'VTermColor',
    'VTermColor_rgb',
    'VTermColor_indexed',
    'VTermStringFragment',
    'VTermValue',
    'VTermGlyphInfo',
    'VTermLineInfo',
    'VTermScreenCellAttrs',
    'VTermScreenCell',
    'VTermPen',
    'ScreenPen',
    'ScreenCell',
    'VTermEncodingInstance',
    'VTermKeyEncodingFlags',
    'VTermKeyEncodingStack',
    'VTermState_mode',
    'VTermState_saved_mode',
    'VTermState_saved',
    'VTermState_tmp_selection',
    'VTermState_tmp',
    'VTermState_selection',
    'VTermStateFields',
    'VTerm_mode',
    'VTerm_parser_v_csi',
    'VTerm_parser_v_osc',
    'VTerm_parser_v_dcs',
    'VTerm_parser_v',
    'VTerm_parser',
    'VTermParserCallbacks',
    'VTermStateCallbacks',
    'VTermStateFallbacks',
    'VTermScreenCallbacks',
    'VTermSelectionCallbacks',
    'VTermState',
    'VTermScreen',
    'VTerm',
  }
  for _, name in ipairs(TYPES) do
    answer('v0/size/' .. name, {
      string.format('sizeof=%d alignof=%d', ffi.sizeof(name), ffi.alignof(name)),
    }, { sizeof = ffi.sizeof(name), alignof = ffi.alignof(name) })
  end

  local OFFS = {
    { 'VTermScreenCell', { 'schar', 'width', 'attrs', 'fg', 'bg', 'uri' } },
    { 'ScreenCell', { 'schar', 'pen' } },
    { 'ScreenPen', { 'fg', 'bg', 'uri' } },
    { 'VTermPen', { 'fg', 'bg', 'uri' } },
    { 'VTermGlyphInfo', { 'schar', 'width' } },
    { 'VTermStringFragment', { 'str', 'terminator' } },
    {
      'VTermState',
      {
        'vt',
        'callbacks',
        'cbdata',
        'fallbacks',
        'fbdata',
        'rows',
        'cols',
        'pos',
        'at_phantom',
        'scrollregion_top',
        'scrollregion_bottom',
        'scrollregion_left',
        'scrollregion_right',
        'tabstops',
        'lineinfos',
        'lineinfo',
        'mouse_col',
        'mouse_row',
        'mouse_buttons',
        'mouse_flags',
        'mouse_protocol',
        'grapheme_buf',
        'grapheme_len',
        'grapheme_last',
        'grapheme_state',
        'combine_width',
        'combine_pos',
        'mode',
        'encoding',
        'encoding_utf8',
        'gl_set',
        'gr_set',
        'gsingle_set',
        'pen',
        'default_fg',
        'default_bg',
        'colors',
        'bold_is_highbright',
        'saved',
        'tmp',
        'selection',
        'key_encoding_stacks',
      },
    },
    {
      'VTermScreen',
      {
        'vt',
        'state',
        'callbacks',
        'cbdata',
        'damage_merge',
        'damaged',
        'pending_scrollrect',
        'pending_scroll_downward',
        'pending_scroll_rightward',
        'rows',
        'cols',
        'buffers',
        'buffer',
        'sb_buffer',
        'pen',
      },
    },
    {
      'VTerm',
      {
        'rows',
        'cols',
        'mode',
        'parser',
        'outfunc',
        'outdata',
        'outbuffer',
        'outbuffer_len',
        'outbuffer_cur',
        'tmpbuffer',
        'tmpbuffer_len',
        'state',
        'screen',
      },
    },
    { 'VTerm_parser', { 'state', 'intermedlen', 'intermed', 'v', 'callbacks', 'cbdata' } },
    { 'VTermState_saved', { 'pos', 'pen', 'mode' } },
    { 'VTermKeyEncodingStack', { 'items', 'size' } },
  }
  for _, entry in ipairs(OFFS) do
    local tname, fields = entry[1], entry[2]
    local parts = {}
    for _, f in ipairs(fields) do
      parts[#parts + 1] = string.format('%s=%d', f, ffi.offsetof(tname, f))
    end
    answer('v0/off/' .. tname, parts)
  end

  -- Live probes.  These are what tell a reader that the cdef above still
  -- describes the binary loaded, rather than a stale copy of it: each one
  -- reads a value back out of the struct after driving the emulator, so
  -- a shifted field answers nonsense rather than a plausible number.
  local t = newterm({ mode = 'screen', rows = 6, cols = 20 })
  t:write(CSI .. '5;7H')
  answer('v0/live/cursor-addressed', {
    string.format('pos=%d,%d rows=%d cols=%d', t.state.pos.row, t.state.pos.col, t.state.rows, t.state.cols),
  })
  t = newterm({ mode = 'screen' })
  t:write(CSI .. '2;5r')
  answer('v0/live/scrollregion', {
    string.format(
      'top=%d bottom=%d left=%d right=%d',
      t.state.scrollregion_top,
      t.state.scrollregion_bottom,
      t.state.scrollregion_left,
      t.state.scrollregion_right
    ),
  })
  t = newterm({ mode = 'screen' })
  t:write('AB' .. CSI .. '1;31mCD')
  local sigs = {}
  for c = 0, 3 do
    POS.row, POS.col = 0, c
    C.vterm_screen_get_cell(t.screen, POS, CELL)
    sigs[#sigs + 1] = cellsig(CELL)
  end
  answer('v0/live/cells', sigs)
  t = newterm({ mode = 'screen' })
  answer('v0/live/vterm-mode', {
    string.format('utf8=%d ctrl8bit=%d rows=%d cols=%d', t.vt.mode.utf8, t.vt.mode.ctrl8bit, t.vt.rows, t.vt.cols),
  })
  t = newterm({ mode = 'screen', utf8 = false })
  answer('v0/live/vterm-mode-nonutf8', {
    string.format('utf8=%d ctrl8bit=%d', t.vt.mode.utf8, t.vt.mode.ctrl8bit),
  })
  t = newterm({ mode = 'parser' })
  t:write(ESC .. '[?')
  answer('v0/live/parser-state', { t:parser_sig() })
  answer('v0/live/encodings', {
    string.format(
      'utf8u=%d s94-0=%d s94-B=%d s94-X=%d s96-A=%d',
      C.vterm_lookup_encoding(0, string.byte('u')) ~= nil and 1 or 0,
      C.vterm_lookup_encoding(1, string.byte('0')) ~= nil and 1 or 0,
      C.vterm_lookup_encoding(1, string.byte('B')) ~= nil and 1 or 0,
      C.vterm_lookup_encoding(1, string.byte('X')) ~= nil and 1 or 0,
      C.vterm_lookup_encoding(2, string.byte('A')) ~= nil and 1 or 0
    ),
  })
end)

-- ============================================================== v1 parser

local function ctrlseq(prefix, body, term)
  return prefix .. body .. term
end

section('v1-parser', function()
  local INPUTS = {}
  local function add(name, bytes)
    INPUTS[#INPUTS + 1] = { name, bytes }
  end

  for b = 0, 31 do
    add(string.format('c0-%02x', b), string.char(b))
  end
  add('c0-7f', string.char(127))
  for b = 0x80, 0x9f do
    add(string.format('c1-%02x', b), string.char(b))
  end
  for _, final in ipairs({
    '7',
    '8',
    '=',
    '>',
    'c',
    'D',
    'E',
    'H',
    'M',
    'N',
    'O',
    'V',
    'W',
    'X',
    'Z',
    '\\',
    'n',
    'o',
    '|',
    '}',
    '~',
  }) do
    add('esc-' .. final, ESC .. final)
  end
  for _, seq in ipairs({
    '(B',
    '(0',
    '(A',
    ')0',
    '*0',
    '+0',
    '-A',
    '.A',
    '/A',
    '#3',
    '#4',
    '#5',
    '#6',
    '#8',
    ' F',
    ' G',
    '%@',
    '%G',
    '!p',
  }) do
    add('esc-int-' .. seq:gsub('%W', function(c)
      return string.format('%02x', c:byte())
    end), ESC .. seq)
  end
  for _, seq in ipairs({
    'H',
    '5H',
    '5;7H',
    ';7H',
    '5;7;9H',
    '?25h',
    '?25l',
    '?1049h',
    '>4;2m',
    '=5u',
    '<1u',
    '0;1;2;3;4;5;6;7;8;9m',
    '38;5;120m',
    '38;2;10;20;30m',
    '38:5:120m',
    '38:2::10:20:30m',
    '4:3m',
    '  q',
    '!p',
    '"q',
    '$p',
    '$|',
    "'}",
    "'~",
    '*x',
    '2 q',
    '1$r',
    '99999999999H',
    '2147483647H',
    '99999999999999999999H',
    '0000000005H',
  }) do
    add('csi-' .. seq:gsub('%W', function(c)
      return string.format('%02x', c:byte())
    end), CSI .. seq)
  end
  add('osc-title-bel', ctrlseq(ESC .. ']', '0;hello', BEL))
  add('osc-title-st', ctrlseq(ESC .. ']', '0;hello', ST))
  add('osc-empty', ctrlseq(ESC .. ']', '', ST))
  add('osc-nocmd', ctrlseq(ESC .. ']', 'plain', ST))
  add('osc-8-link', ctrlseq(ESC .. ']', '8;;http://a/b', ST))
  add('osc-52', ctrlseq(ESC .. ']', '52;c;aGVsbG8=', ST))
  add('osc-huge-cmd', ctrlseq(ESC .. ']', '99999;x', ST))
  add('osc-c1', ctrlseq('\157', '0;c1', ST))
  add('dcs-plain', ctrlseq(ESC .. 'P', '1$qm', ST))
  add('dcs-empty', ctrlseq(ESC .. 'P', '', ST))
  add('dcs-long-cmd', ctrlseq(ESC .. 'P', 'abcdefghijklmnopqrstuvwxyz', ST))
  add('dcs-c1', ctrlseq('\144', '1$qr', ST))
  add('apc', ctrlseq(ESC .. '_', 'apcdata', ST))
  add('pm', ctrlseq(ESC .. '^', 'pmdata', ST))
  add('sos', ctrlseq(ESC .. 'X', 'sosdata', ST))
  add('apc-c1', ctrlseq('\159', 'apc', ST))
  add('text-plain', 'hello')
  add('text-del', 'a\127b')
  add('text-nul', 'a\0b')
  add('text-high', 'a\200\201b')
  add('text-utf8', 'a\226\130\172b')
  add('text-esc-mid', 'ab' .. ESC .. 'Dcd')
  add('csi-in-text', 'ab' .. CSI .. '2Jcd')
  add('esc-csi-cancel', ESC .. '[1;2' .. ESC .. 'D')
  add('csi-can', CSI .. '1;2\24H')
  add('csi-sub', CSI .. '1;2\26H')
  add('osc-can', ESC .. ']0;ab\24cd' .. ST)
  add('c1-in-csi', CSI .. '1\155' .. '2H')

  local SPLITS = { 'whole', 'bytes', 'half', 'edges' }
  for _, inp in ipairs(INPUTS) do
    for _, how in ipairs(SPLITS) do
      if how == 'whole' or #inp[2] > 1 then
        local t = newterm({ mode = 'parser' })
        local n = t:writesplit(inp[2], how)
        parser_answer('v1/' .. inp[1] .. '/' .. how, t, 'n=' .. n)
      end
    end
  end

  -- The 8-bit C1 set is only C1 when the terminal is not decoding UTF-8.
  for b = 0x80, 0x9f do
    local t = newterm({ mode = 'parser', utf8 = false })
    local n = t:write(string.char(b))
    parser_answer(string.format('v1/c1-nonutf8-%02x', b), t, 'n=' .. n)
  end
end)

-- ================================================================= v2 CSI

local function paint(t)
  t:write(CSI .. 'H')
  for r = 1, t.rows do
    t:write(string.rep(string.char(64 + r), t.cols - 4))
    if r < t.rows then
      t:write('\r\n')
    end
  end
  t:write(CSI .. '3;5H')
  t:reset()
end

section('v2-csi', function()
  local FINALS = {
    'A',
    'B',
    'C',
    'D',
    'E',
    'F',
    'G',
    'H',
    'I',
    'J',
    'K',
    'L',
    'M',
    'P',
    'S',
    'T',
    'X',
    'Z',
    '@',
    '`',
    'a',
    'b',
    'd',
    'e',
    'f',
    'j',
    'k',
    'n',
    'x',
  }
  local ARGS = {
    '',
    '0',
    '1',
    '2',
    '3',
    '5',
    '99',
    '0;0',
    '2;3',
    '9;99',
    '2147483647',
    '1;2;3;4',
  }
  for _, final in ipairs(FINALS) do
    for _, args in ipairs(ARGS) do
      local t = newterm({ mode = 'screen' })
      paint(t)
      t:write(CSI .. args .. final)
      screen_answer(
        string.format('v2/plain/%02x/%s', final:byte(), args == '' and 'none' or args:gsub(';', '_')),
        t
      )
    end
  end
  -- The same commands inside a scroll region and an origin-mode window,
  -- which is where the clamps live.
  local REGION = { 'D', 'E', 'F', 'L', 'M', 'S', 'T', 'J', 'K', 'H' }
  for _, final in ipairs(REGION) do
    for _, args in ipairs(ARGS) do
      local t = newterm({ mode = 'screen' })
      paint(t)
      t:write(CSI .. '2;5r' .. CSI .. '?6h' .. CSI .. '3;5H')
      t:reset()
      t:write(CSI .. args .. final)
      screen_answer(
        string.format('v2/region/%02x/%s', final:byte(), args == '' and 'none' or args:gsub(';', '_')),
        t
      )
    end
  end
  -- Private-parameter erases (DECSED/DECSEL) and the selective flavour.
  for _, seq in ipairs({ '?0J', '?1J', '?2J', '?3J', '?0K', '?1K', '?2K', '0J', '1J', '2J', '3J' }) do
    local t = newterm({ mode = 'screen' })
    paint(t)
    t:write(CSI .. '?1' .. string.char(34) .. 'q') -- DECSCA: protect
    t:write('PP')
    t:write(CSI .. '3;5H')
    t:reset()
    t:write(CSI .. seq)
    screen_answer('v2/erase/' .. seq:gsub('%?', 'q'), t)
  end
  -- Left/right margins need DECLRMM on first.
  for _, seq in ipairs({ '1;10s', '5;5s', '0;0s', '10;1s', '1;99s', 's' }) do
    local t = newterm({ mode = 'screen' })
    paint(t)
    t:write(CSI .. '?69h' .. CSI .. seq)
    screen_answer('v2/margin/' .. seq:gsub(';', '_'), t)
  end
  -- WRAPPED content, so that the CONTINUATION marks an erase cancels
  -- actually exist.  `paint()` writes rows that stop four columns short
  -- of the right edge and therefore sets NONE of them, which left
  -- `VTermState::erase`'s whole continuation arm ungated: every erase
  -- row above answers `000:` for every line whatever the arm does.
  -- The mark is only visible in the per-row `lineinfo` prefix of the
  -- grid answer, so it needs content that really did wrap.
  for _, at in ipairs({ '1;1H', '2;1H', '3;1H', '4;1H' }) do
    for _, seq in ipairs({
      '0K',
      '1K',
      '2K',
      '0J',
      '1J',
      '2J',
      '3J',
      '5X',
      '20X',
      '2P',
      '2M',
      '2L',
    }) do
      local t = newterm({ mode = 'screen' })
      t:write(string.rep('x', 70))
      t:write(CSI .. at)
      t:reset()
      t:write(CSI .. seq)
      screen_answer('v2/erasecont/' .. at:gsub(';', '_') .. '/' .. seq, t)
    end
  end
  -- Tab stops.
  for _, seq in ipairs({ 'H', '0g', '3g', '5g', '2I', '2Z', '9I' }) do
    local t = newterm({ mode = 'screen' })
    paint(t)
    t:write('\t\t' .. ESC .. 'H' .. CSI .. seq .. '\t')
    screen_answer('v2/tab/' .. seq, t)
  end
end)

-- ================================================================= v3 SGR

section('v3-sgr', function()
  local SINGLE = {}
  for _, n in ipairs({
    0,
    1,
    2,
    3,
    4,
    5,
    6,
    7,
    8,
    9,
    10,
    11,
    12,
    20,
    21,
    22,
    23,
    24,
    25,
    26,
    27,
    28,
    29,
    39,
    49,
    50,
    51,
    52,
    53,
    54,
    55,
    58,
    59,
    73,
    74,
    75,
    99,
  }) do
    SINGLE[#SINGLE + 1] = tostring(n)
  end
  for n = 30, 38 do
    SINGLE[#SINGLE + 1] = tostring(n)
  end
  for n = 40, 48 do
    SINGLE[#SINGLE + 1] = tostring(n)
  end
  for n = 90, 97 do
    SINGLE[#SINGLE + 1] = tostring(n)
  end
  for n = 100, 107 do
    SINGLE[#SINGLE + 1] = tostring(n)
  end
  for _, s in ipairs({
    '38;5;0',
    '38;5;15',
    '38;5;120',
    '38;5;255',
    '38;5;256',
    '38;5',
    '38;2;0;0;0',
    '38;2;10;20;30',
    '38;2;255;255;255',
    '38;2;300;20;30',
    '38;2;10;20',
    '38;3;1;2;3',
    '38;9;1',
    '48;5;120',
    '48;2;10;20;30',
    '58;5;3',
    '58;2;1;2;3',
    '38:5:120',
    '38:2::10:20:30',
    '38:2:0:10:20:30',
    '48:5:9',
    '58:5:9',
    '4:0',
    '4:1',
    '4:2',
    '4:3',
    '4:4',
    '4:5',
    '4:9',
    '1;4;7',
    '0;1',
    '1;22',
    '4;24',
    '7;27',
    '30;41;1;4',
    '39;49',
    '1;2',
    '53;55',
  }) do
    SINGLE[#SINGLE + 1] = s
  end
  -- `;` and `:` must NOT collapse to the same label character: `38;5;120`
  -- and `38:5:120` are different sequences and would otherwise be one row.
  local function sgrlabel(s)
    return (s:gsub(';', '_'):gsub(':', 'c'))
  end
  for _, s in ipairs(SINGLE) do
    local t = newterm({ mode = 'state' })
    t:write(CSI .. s .. 'm')
    state_answer('v3/sgr/' .. sgrlabel(s), t)
  end
  -- Order matters: an attribute set then partially cleared.
  local PAIRS = { '1', '2', '4', '7', '9', '30', '38;5;9', '48;2;1;2;3', '53', '4:3' }
  for _, a in ipairs(PAIRS) do
    for _, b in ipairs(PAIRS) do
      local t = newterm({ mode = 'state' })
      t:write(CSI .. a .. 'm' .. CSI .. b .. 'm')
      state_answer('v3/pair/' .. sgrlabel(a) .. '+' .. sgrlabel(b), t)
    end
  end
  -- DECRQSS read-back: the reply is the pen the emulator believes in.
  for _, req in ipairs({ 'm', 'r', 's', ' q', '"q', '"p', '$}', '$~', 'x', 'zz' }) do
    for _, pre in ipairs({
      { 'clean', '' },
      { 'pen', CSI .. '1;4;38;5;9m' },
      { 'region', CSI .. '2;7r' .. CSI .. '3 q' },
    }) do
      local t = newterm({ mode = 'state' })
      t:write(pre[2])
      t:reset()
      t:write(ESC .. 'P$q' .. req .. ST)
      state_answer(
        'v3/decrqss/' .. req:gsub('%W', function(c)
          return string.format('%02x', c:byte())
        end) .. '/' .. pre[1],
        t
      )
    end
  end
  -- The bold-is-highbright flag is only reachable through the palette.
  for _, seq in ipairs({ '1;30', '1;90', '1;38;5;1', '22;30' }) do
    local t = newterm({ mode = 'screen' })
    t:write(CSI .. seq .. 'mX')
    screen_answer('v3/highbright/' .. seq:gsub(';', '_'), t)
  end
end)

-- =============================================================== v4 modes

section('v4-modes', function()
  local DEC = {
    1,
    2,
    3,
    4,
    5,
    6,
    7,
    8,
    9,
    12,
    25,
    47,
    66,
    69,
    80,
    1000,
    1002,
    1003,
    1004,
    1005,
    1006,
    1015,
    1047,
    1048,
    1049,
    2004,
    2026,
    2031,
    9999,
  }
  for _, m in ipairs(DEC) do
    for _, set in ipairs({ 'h', 'l' }) do
      local t = newterm({ mode = 'screen', altscreen = true })
      t:write('seed')
      t:reset()
      t:write(CSI .. '?' .. m .. set)
      screen_answer(string.format('v4/dec/%d%s', m, set), t)
    end
  end
  for _, m in ipairs({ 2, 4, 12, 20, 34, 9999 }) do
    for _, set in ipairs({ 'h', 'l' }) do
      local t = newterm({ mode = 'screen' })
      t:write(CSI .. m .. set)
      screen_answer(string.format('v4/ansi/%d%s', m, set), t)
    end
  end
  -- DECSTBM: every degenerate region the clamp has to reject or fix up.
  for _, args in ipairs({
    '',
    '1;6',
    '2;5',
    '5;2',
    '0;0',
    '1;1',
    '6;6',
    '1;99',
    '99;1',
    '3',
    ';4',
    '2;',
    '0;3',
    '2147483647;1',
  }) do
    local t = newterm({ mode = 'screen' })
    paint(t)
    t:write(CSI .. args .. 'r')
    screen_answer('v4/decstbm/' .. (args == '' and 'none' or args:gsub(';', '_')), t)
  end
  -- Origin mode changes what every address means.
  for _, seq in ipairs({ '?6h', '?6l' }) do
    for _, addr in ipairs({ 'H', '1;1H', '3;3H', '99;99H', '5d', '9d' }) do
      local t = newterm({ mode = 'screen' })
      paint(t)
      t:write(CSI .. '2;5r' .. CSI .. seq)
      t:reset()
      t:write(CSI .. addr)
      screen_answer('v4/origin/' .. seq:gsub('%?', 'q') .. '/' .. addr:gsub(';', '_'), t)
    end
  end
  -- Autowrap and the phantom column.
  for _, seq in ipairs({ '?7h', '?7l' }) do
    for _, extra in ipairs({ { 'plain', '' }, { 'irm', CSI .. '4h' }, { 'rvwrap', CSI .. '?45h' } }) do
      local t = newterm({ mode = 'screen', rows = 4, cols = 8 })
      t:write(CSI .. seq .. extra[2] .. CSI .. '1;6H')
      t:reset()
      t:write('abcdefgh')
      screen_answer('v4/wrap/' .. seq:gsub('%?', 'q') .. '/' .. extra[1], t)
    end
  end
  -- Save/restore cursor and DECALN.
  for _, seq in ipairs({
    ESC .. '7',
    ESC .. '8',
    ESC .. '7' .. CSI .. '1;1H' .. ESC .. '8',
    CSI .. 's' .. CSI .. '1;1H' .. CSI .. 'u',
    ESC .. '#8',
    ESC .. '#3',
    ESC .. '#4',
    ESC .. '#5',
    ESC .. '#6',
  }) do
    local t = newterm({ mode = 'screen' })
    paint(t)
    t:write(seq)
    screen_answer(
      'v4/save/' .. seq:gsub('%W', function(c)
        return string.format('%02x', c:byte())
      end),
      t
    )
  end
  -- The kitty key-encoding stack: push, pop, set, query, overflow.
  local KITTY = {
    '>u',
    '>1u',
    '>31u',
    '>99u',
    '=1u',
    '=1;1u',
    '=1;2u',
    '=1;3u',
    '=31;1u',
    '<u',
    '<1u',
    '<99u',
    '?u',
  }
  for _, seq in ipairs(KITTY) do
    for _, depth in ipairs({ 0, 1, 17 }) do
      local t = newterm({ mode = 'state' })
      for _ = 1, depth do
        t:write(CSI .. '>5u')
      end
      t:reset()
      t:write(CSI .. seq)
      state_answer(
        'v4/kitty/' .. seq:gsub('%W', function(c)
          return string.format('%02x', c:byte())
        end) .. '/d' .. depth,
        t
      )
    end
  end
  -- Cursor shape, blink and the focus reports.
  for n = 0, 7 do
    local t = newterm({ mode = 'screen' })
    t:write(CSI .. n .. ' q')
    screen_answer('v4/decscusr/' .. n, t)
  end
  for _, which in ipairs({ 'in', 'out' }) do
    for _, pre in ipairs({ { 'off', '' }, { 'on', CSI .. '?1004h' } }) do
      local t = newterm({ mode = 'screen' })
      t:write(pre[2])
      t:reset()
      if which == 'in' then
        C.vterm_state_focus_in(t.state)
      else
        C.vterm_state_focus_out(t.state)
      end
      screen_answer('v4/focus/' .. which .. '/' .. pre[1], t)
    end
  end
end)

-- ================================================================ v5 UTF-8

section('v5-utf8', function()
  local STRINGS = {
    { 'ascii', 'abc' },
    { 'cjk', '\228\184\173\230\150\135' },
    { 'cjk-wrap', string.rep('\228\184\173', 12) },
    { 'combining', 'e\204\129' },
    { 'combining-many', 'a\204\129\204\130\204\131\204\132' },
    { 'combining-first', '\204\129abc' },
    { 'zwj', '\240\159\145\168\226\128\141\240\159\146\187' },
    { 'flag', '\240\159\135\186\240\159\135\184' },
    { 'emoji-vs16', '\226\152\186\239\184\143' },
    { 'emoji-plain', '\240\159\152\128' },
    { 'skintone', '\240\159\145\141\240\159\143\189' },
    { 'nbsp', 'a\194\160b' },
    { 'zwsp', 'a\226\128\139b' },
    { 'rtl', '\215\144\215\145' },
    { 'thai', '\224\184\129\224\184\181' },
    { 'hangul-jamo', '\225\132\128\225\133\161' },
    { 'twobyte', '\194\169' },
    { 'threebyte', '\226\130\172' },
    { 'fourbyte', '\240\157\132\158' },
    { 'trunc2', '\194' },
    { 'trunc3', '\226\130' },
    { 'trunc4', '\240\157\132' },
    { 'trunc-then-ascii', '\226\130Z' },
    { 'overlong2', '\192\175' },
    { 'overlong3', '\224\128\175' },
    { 'overlong4', '\240\128\128\175' },
    { 'surrogate', '\237\160\128' },
    { 'maxplus', '\244\144\128\128' },
    { 'fe', '\254' },
    { 'ff', '\255' },
    { 'bare-cont', '\128\129' },
    { 'c1-8f', '\194\143' },
    { 'c1-9b', '\194\155' },
    { 'nul-mid', 'a\0b' },
    { 'del-mid', 'a\127b' },
    { 'bs-combine', 'a\8\204\129' },
    { 'tab', 'a\tb' },
    { 'cr', 'ab\rc' },
    { 'lf', 'ab\nc' },
    { 'crlf', 'ab\r\nc' },
    { 'ff-c0', 'ab\12c' },
    { 'vt', 'ab\11c' },
    { 'bel', 'ab\7c' },
    { 'bs-at-col0', '\8ab' },
    { 'wide-at-edge', string.rep('x', 19) .. '\228\184\173' },
    { 'wide-then-combining', '\228\184\173\204\129' },
    { 'long-ascii', string.rep('abcdefghij', 3) },
    { 'high-latin1', '\233\224\231' },
    { 'mixed', 'a\228\184\173b\204\129c' },
    { 'reverse-solidus', 'a\\b' },
    { 'space-only', '   ' },
    { 'dec-line', 'lqk' },
  }
  for _, s in ipairs(STRINGS) do
    for _, utf8 in ipairs({ true, false }) do
      local t = newterm({ mode = 'screen', utf8 = utf8, rows = 4, cols = 20 })
      t:write(s[2])
      screen_answer('v5/str/' .. s[1] .. '/' .. (utf8 and 'u8' or 'raw'), t)
    end
  end
  -- RECOMBINATION ACROSS AN `input_write` BOUNDARY.  Inside one write
  -- `text::print` gathers a cluster in its own loop; the
  -- `combine_pos`/`combine_width` window at the top of `print` is only
  -- reached when a call STARTS with a character that belongs to the
  -- glyph the PREVIOUS call left behind.  Every case above hands the
  -- whole string over at once, so that window was ungated.
  local SPLITSTR = {
    { 'e-acute', 'e\204\129' },
    { 'a-many', 'a\204\129\204\130\204\131' },
    { 'cjk-comb', '\228\184\173\204\129' },
    { 'zwj', '\240\159\145\168\226\128\141\240\159\146\187' },
    { 'ascii-run', 'abc' },
    { 'wide-run', '\228\184\173\228\184\173' },
    { 'comb-after-wrap', string.rep('y', 19) .. '\228\184\173\204\129' },
    { 'comb-at-start', '\204\129x' },
    { 'comb-after-cr', 'e\r\204\129' },
    { 'comb-after-move', 'e' .. CSI .. '1;1H' .. '\204\129' },
  }
  for _, s in ipairs(SPLITSTR) do
    for _, how in ipairs({ 'whole', 'bytes', 'half', 'edges' }) do
      local t = newterm({ mode = 'screen', rows = 4, cols = 20 })
      t:writesplit(s[2], how)
      screen_answer('v5/split/' .. s[1] .. '/' .. how, t)
    end
  end
  -- Charset designation, locking shifts and single shifts.
  local DESIGNATE = {
    '(B',
    '(0',
    '(A',
    '(U',
    '(<',
    ')B',
    ')0',
    '*0',
    '+0',
    '-A',
    '.A',
    '/A',
    '%@',
    '%G',
  }
  local SAMPLES = { { 'line', 'lqkx' }, { 'alpha', 'ABC' }, { 'cjk', '\228\184\173' } }
  for _, d in ipairs(DESIGNATE) do
    for _, sample in ipairs(SAMPLES) do
      local t = newterm({ mode = 'screen', rows = 4, cols = 20 })
      t:write(ESC .. d)
      t:reset()
      t:write(sample[2])
      screen_answer(
        'v5/designate/' .. d:gsub('%W', function(c)
          return string.format('%02x', c:byte())
        end) .. '/' .. sample[1],
        t
      )
    end
  end
  for _, shift in ipairs({ '\14', '\15', ESC .. 'N', ESC .. 'O', ESC .. 'n', ESC .. 'o', ESC .. '~', ESC .. '}', ESC .. '|' }) do
    local t = newterm({ mode = 'screen', rows = 4, cols = 20 })
    t:write(ESC .. ')0')
    t:reset()
    t:write(shift .. 'lqk')
    screen_answer(
      'v5/shift/' .. shift:gsub('%W', function(c)
        return string.format('%02x', c:byte())
      end),
      t
    )
  end
end)

-- ================================================================== v6 OSC

section('v6-osc', function()
  local OSCS = {
    { 'title', '0;a title' },
    { 'iconname', '1;icon' },
    { 'title2', '2;window title' },
    { 'title-empty', '0;' },
    { 'title-utf8', '0;\228\184\173' },
    { 'title-ctrl', '0;a\7b' },
    { 'palette-set', '4;1;#ff0000' },
    { 'palette-set-rgb', '4;2;rgb:00/ff/00' },
    { 'palette-query', '4;3;?' },
    { 'palette-reset', '104;1' },
    { 'palette-reset-all', '104' },
    { 'fg-set', '10;#123456' },
    { 'fg-query', '10;?' },
    { 'bg-set', '11;#654321' },
    { 'bg-query', '11;?' },
    { 'cursor-color', '12;#ffffff' },
    { 'fg-reset', '110' },
    { 'bg-reset', '111' },
    { 'cursor-reset', '112' },
    { 'hyperlink', '8;;http://example/' },
    { 'hyperlink-id', '8;id=xy;http://example/' },
    { 'hyperlink-close', '8;;' },
    { 'sel-set-c', '52;c;aGVsbG8=' },
    { 'sel-set-p', '52;p;aGVsbG8=' },
    { 'sel-set-s', '52;s;d29ybGQ=' },
    { 'sel-set-multi', '52;cp;aGVsbG8=' },
    { 'sel-query', '52;c;?' },
    { 'sel-clear', '52;c;' },
    { 'sel-bad-b64', '52;c;!!!' },
    { 'sel-cut0', '52;0;YQ==' },
    { 'shell-prompt', '133;A' },
    { 'unknown', '777;notify;a;b' },
    { 'nonnumeric', 'zz;payload' },
    { 'negative', '-1;x' },
    { 'huge', '4294967296;x' },
  }
  for _, o in ipairs(OSCS) do
    for _, term in ipairs({ 'bel', 'st' }) do
      for _, how in ipairs({ 'whole', 'bytes' }) do
        local t = newterm({ mode = 'screen', selection = true })
        t:writesplit(ESC .. ']' .. o[2] .. (term == 'bel' and BEL or ST), how)
        screen_answer('v6/osc/' .. o[1] .. '/' .. term .. '/' .. how, t)
      end
    end
  end
  -- Unterminated and re-entered control strings.
  for _, name_seq in ipairs({
    { 'osc-unterminated', ESC .. ']0;abc' },
    { 'osc-then-text', ESC .. ']0;abc' .. ST .. 'XY' },
    { 'osc-nested-esc', ESC .. ']0;a' .. ESC .. 'Db' .. ST },
    { 'osc-8bit-st', ESC .. ']0;abc\156' },
    { 'dcs-decrqss-m', ESC .. 'P$qm' .. ST },
    { 'dcs-decrqss-r', ESC .. 'P$qr' .. ST },
    { 'dcs-decrqss-sp-q', ESC .. 'P$q q' .. ST },
    { 'dcs-decrqss-bad', ESC .. 'P$qZZ' .. ST },
    { 'dcs-unknown', ESC .. 'Pxyz' .. ST },
    { 'dcs-empty', ESC .. 'P' .. ST },
    { 'dcs-long', ESC .. 'P' .. string.rep('a', 40) .. ST },
    { 'apc-plain', ESC .. '_payload' .. ST },
    { 'pm-plain', ESC .. '^payload' .. ST },
    { 'sos-plain', ESC .. 'Xpayload' .. ST },
    { 'apc-empty', ESC .. '_' .. ST },
    { 'sos-with-esc', ESC .. 'Xa' .. ESC .. 'b' .. ST },
  }) do
    for _, how in ipairs({ 'whole', 'bytes', 'half' }) do
      local t = newterm({ mode = 'state', fallbacks = true, selection = true })
      t:writesplit(name_seq[2], how)
      state_answer('v6/ctrlstr/' .. name_seq[1] .. '/' .. how, t)
    end
  end
  -- Selection with and without a consumer, and with a tiny staging buffer.
  for _, buflen in ipairs({ 0, 4, 16, 128 }) do
    for _, payload in ipairs({
      { 'short', 'YQ==' },
      { 'medium', 'aGVsbG8gd29ybGQ=' },
      { 'long', string.rep('YQ==', 12) },
    }) do
      local t = newterm({ mode = 'state', selection = true, selbuflen = buflen })
      t:write(ESC .. ']52;c;' .. payload[2] .. ST)
      state_answer('v6/sel/' .. buflen .. '/' .. payload[1], t)
    end
  end
end)

-- ================================================================ v7 mouse

section('v7-mouse', function()
  local PROTOS = {
    { 'x10', '' },
    { 'utf8', CSI .. '?1005h' },
    { 'rxvt', CSI .. '?1015h' },
    { 'sgr', CSI .. '?1006h' },
  }
  local MODES = { { 'click', '?1000h' }, { 'drag', '?1002h' }, { 'move', '?1003h' }, { 'off', '?1000l' } }
  local MODS = {
    { 'none', MOD.NONE },
    { 'shift', MOD.SHIFT },
    { 'alt', MOD.ALT },
    { 'ctrl', MOD.CTRL },
    { 'all', 7 },
  }
  for _, proto in ipairs(PROTOS) do
    for _, mode in ipairs(MODES) do
      for _, m in ipairs(MODS) do
        for _, btn in ipairs({ 1, 2, 3, 4, 5, 8 }) do
          local t = newterm({ mode = 'state' })
          t:write(CSI .. mode[2] .. proto[2])
          C.vterm_mouse_move(t.vt, 2, 3, MOD.NONE)
          t:reset()
          C.vterm_mouse_button(t.vt, btn, true, m[2])
          C.vterm_mouse_button(t.vt, btn, false, m[2])
          state_answer(
            string.format('v7/btn/%s/%s/%s/%d', proto[1], mode[1], m[1], btn),
            t,
            'mouse=' .. t.state.mouse_buttons
          )
        end
        local t = newterm({ mode = 'state' })
        t:write(CSI .. mode[2] .. proto[2])
        C.vterm_mouse_move(t.vt, 1, 1, MOD.NONE)
        t:reset()
        C.vterm_mouse_move(t.vt, 4, 9, m[2])
        state_answer(string.format('v7/move/%s/%s/%s', proto[1], mode[1], m[1]), t)

        local d = newterm({ mode = 'state' })
        d:write(CSI .. mode[2] .. proto[2])
        C.vterm_mouse_move(d.vt, 1, 1, MOD.NONE)
        C.vterm_mouse_button(d.vt, 1, true, MOD.NONE)
        d:reset()
        C.vterm_mouse_move(d.vt, 3, 7, m[2])
        C.vterm_mouse_button(d.vt, 1, false, m[2])
        state_answer(string.format('v7/drag/%s/%s/%s', proto[1], mode[1], m[1]), d)
      end
    end
    -- Positions: the encodings differ in exactly how they run out of room.
    for _, p in ipairs({
      { 0, 0 },
      { 1, 1 },
      { 32, 32 },
      { 93, 93 },
      { 94, 94 },
      { 200, 200 },
      { 222, 222 },
      { 223, 223 },
      { 1000, 1000 },
      { 65535, 65535 },
    }) do
      local t = newterm({ mode = 'state', rows = 6, cols = 20 })
      t:write(CSI .. '?1003h' .. proto[2])
      t:reset()
      C.vterm_mouse_move(t.vt, p[1], p[2], MOD.NONE)
      C.vterm_mouse_button(t.vt, 1, true, MOD.NONE)
      state_answer(string.format('v7/pos/%s/%d_%d', proto[1], p[1], p[2]), t)
    end
  end
  -- Protocol selection order: 1005/1006/1015 are mutually exclusive, and
  -- turning one off does not restore the one it displaced.
  local ORDERS = {
    { '1005then1006', { '?1005h', '?1006h' } },
    { '1006then1005', { '?1006h', '?1005h' } },
    { '1015then1006', { '?1015h', '?1006h' } },
    { '1006then1015', { '?1006h', '?1015h' } },
    { '1006on-off', { '?1006h', '?1006l' } },
    { '1005on-off', { '?1005h', '?1005l' } },
    { '1015on-off', { '?1015h', '?1015l' } },
    { 'all-on', { '?1005h', '?1006h', '?1015h' } },
    { 'all-off', { '?1005h', '?1006h', '?1015h', '?1005l', '?1006l', '?1015l' } },
  }
  for _, order in ipairs(ORDERS) do
    local t = newterm({ mode = 'state' })
    t:write(CSI .. '?1000h')
    for _, seq in ipairs(order[2]) do
      t:write(CSI .. seq)
    end
    t:reset()
    C.vterm_mouse_move(t.vt, 5, 5, MOD.NONE)
    C.vterm_mouse_button(t.vt, 1, true, MOD.NONE)
    state_answer('v7/proto/' .. order[1], t)
  end
end)

-- ============================================================= v8 keyboard

section('v8-keyboard', function()
  local KEYS = {}
  for _, name in ipairs({
    'NONE',
    'ENTER',
    'TAB',
    'BACKSPACE',
    'ESCAPE',
    'UP',
    'DOWN',
    'LEFT',
    'RIGHT',
    'INS',
    'DEL',
    'HOME',
    'END',
    'PAGEUP',
    'PAGEDOWN',
  }) do
    KEYS[#KEYS + 1] = { name, KEY[name] }
  end
  for n = 1, 20 do
    KEYS[#KEYS + 1] = { 'F' .. n, KEY.FUNCTION_0 + n }
  end
  local KPNAMES = {
    '0',
    '1',
    '2',
    '3',
    '4',
    '5',
    '6',
    '7',
    '8',
    '9',
    'MULT',
    'PLUS',
    'COMMA',
    'MINUS',
    'PERIOD',
    'DIVIDE',
    'ENTER',
    'EQUAL',
  }
  for i, name in ipairs(KPNAMES) do
    KEYS[#KEYS + 1] = { 'KP_' .. name, KEY.KP_0 + i - 1 }
  end
  KEYS[#KEYS + 1] = { 'PAST_END', 1024 }

  local MODECOMBOS = {
    { 'plain', '' },
    { 'ckm', CSI .. '?1h' },
    { 'kpam', ESC .. '=' },
    { 'both', CSI .. '?1h' .. ESC .. '=' },
  }
  for _, combo in ipairs(MODECOMBOS) do
    for _, key in ipairs(KEYS) do
      for m = 0, 7 do
        local t = newterm({ mode = 'state' })
        t:write(combo[2])
        t:reset()
        C.vterm_keyboard_key(t.vt, key[2], m)
        state_answer(string.format('v8/key/%s/%s/m%d', combo[1], key[1], m), t)
      end
    end
  end
  local CHARS = {
    { 'a', 97 },
    { 'A', 65 },
    { 'z', 122 },
    { 'sp', 32 },
    { 'at', 64 },
    { 'lbr', 91 },
    { 'bsl', 92 },
    { 'rbr', 93 },
    { 'caret', 94 },
    { 'us', 95 },
    { 'grave', 96 },
    { 'q', 63 },
    { 'digit0', 48 },
    { 'digit9', 57 },
    { 'nul', 0 },
    { 'tab', 9 },
    { 'cr', 13 },
    { 'esc', 27 },
    { 'del', 127 },
    { 'eacute', 0xe9 },
    { 'cjk', 0x4e2d },
    { 'emoji', 0x1f600 },
    { 'max', 0x10ffff },
    { 'past-max', 0x110000 },
  }
  for _, combo in ipairs({ { 'plain', '' }, { 'lnm', CSI .. '20h' }, { 'kitty', CSI .. '>1u' } }) do
    for _, ch in ipairs(CHARS) do
      for m = 0, 7 do
        local t = newterm({ mode = 'state' })
        t:write(combo[2])
        t:reset()
        C.vterm_keyboard_unichar(t.vt, ch[2], m)
        state_answer(string.format('v8/uni/%s/%s/m%d', combo[1], ch[1], m), t)
      end
    end
  end
  for _, pre in ipairs({ { 'default', '' }, { 'bp-on', CSI .. '?2004h' }, { 'bp-off', CSI .. '?2004l' } }) do
    local t = newterm({ mode = 'state' })
    t:write(pre[2])
    t:reset()
    C.vterm_keyboard_start_paste(t.vt)
    C.vterm_keyboard_unichar(t.vt, 97, 0)
    C.vterm_keyboard_end_paste(t.vt)
    state_answer('v8/paste/' .. pre[1], t)
  end
end)

-- =============================================================== v9 damage

section('v9-damage', function()
  -- The one hole `vterm_spec.lua` leaves: upstream's `62screen_damage`
  -- is `pending()`.  Every merge level x a scripted paint, dumping every
  -- rect the callback saw AND the residual merge bookkeeping, which is
  -- where an unflushed rect lives (`vterm_screen_flush_damage` is not an
  -- exported symbol, so the residual IS the answer).
  local SCRIPTS = {
    { 'one-cell', 'A' },
    { 'one-row', 'ABCDEFGH' },
    { 'two-rows', 'ABCD\r\nEFGH' },
    { 'all-rows', 'A\r\nB\r\nC\r\nD\r\nE\r\nF' },
    { 'row-then-far-row', 'AAAA' .. CSI .. '5;1H' .. 'BBBB' },
    { 'far-row-then-row', CSI .. '5;1H' .. 'BBBB' .. CSI .. '1;1H' .. 'AAAA' },
    { 'same-row-gap', 'AA' .. CSI .. '1;15H' .. 'BB' },
    { 'col-walk', 'A' .. CSI .. '1;10H' .. 'B' .. CSI .. '1;20H' .. 'C' },
    { 'erase-all', 'ABC' .. CSI .. '2J' },
    { 'erase-line', 'ABC' .. CSI .. '2K' },
    { 'erase-to-end', 'ABCDEF' .. CSI .. '1;3H' .. CSI .. '0K' },
    { 'erase-to-start', 'ABCDEF' .. CSI .. '1;3H' .. CSI .. '1K' },
    { 'erase-display-below', 'A\r\nB\r\nC' .. CSI .. '2;1H' .. CSI .. '0J' },
    { 'erase-display-above', 'A\r\nB\r\nC' .. CSI .. '2;1H' .. CSI .. '1J' },
    { 'scroll-up-one', 'A\r\nB\r\nC\r\nD\r\nE\r\nF\r\nG' },
    { 'scroll-up-many', string.rep('X\r\n', 12) },
    { 'scroll-region-up', CSI .. '2;5r' .. CSI .. '2;1H' .. string.rep('Y\r\n', 6) },
    { 'insert-line', 'A\r\nB\r\nC' .. CSI .. '2;1H' .. CSI .. '2L' },
    { 'delete-line', 'A\r\nB\r\nC' .. CSI .. '1;1H' .. CSI .. '1M' },
    { 'insert-char', 'ABCDEF' .. CSI .. '1;3H' .. CSI .. '3@' },
    { 'delete-char', 'ABCDEF' .. CSI .. '1;3H' .. CSI .. '3P' },
    { 'erase-char', 'ABCDEF' .. CSI .. '1;3H' .. CSI .. '3X' },
    { 'scroll-up-su', 'A\r\nB\r\nC' .. CSI .. '2S' },
    { 'scroll-down-sd', 'A\r\nB\r\nC' .. CSI .. '2T' },
    { 'scroll-left', 'ABCDEF' .. CSI .. '?69h' .. CSI .. '3 @' },
    { 'scroll-right', 'ABCDEF' .. CSI .. '?69h' .. CSI .. '3 A' },
    { 'reverse-index', CSI .. '1;1H' .. ESC .. 'M' .. ESC .. 'M' },
    { 'index-at-bottom', CSI .. '6;1H' .. ESC .. 'D' .. ESC .. 'D' },
    { 'wide-glyph', '\228\184\173\228\184\173' },
    { 'combining', 'e\204\129f' },
    { 'wrap', string.rep('x', 25) },
    { 'wrap-wide', string.rep('\228\184\173', 11) },
    { 'decaln', ESC .. '#8' },
    { 'reverse-video', 'ABC' .. CSI .. '?5h' .. 'DEF' },
    { 'reverse-video-off', 'ABC' .. CSI .. '?5h' .. CSI .. '?5l' },
    { 'altscreen-in', 'ABC' .. CSI .. '?1049h' .. 'DEF' },
    { 'altscreen-out', 'ABC' .. CSI .. '?1049h' .. 'DEF' .. CSI .. '?1049l' },
    { 'sgr-then-paint', CSI .. '1;31m' .. 'ABC' },
    { 'cursor-only', CSI .. '3;7H' },
    { 'cursor-then-cell', CSI .. '3;7H' .. 'Z' },
    { 'bell', 'A\7B' },
    { 'tab-fill', 'A\tB\tC' },
    { 'cr-overwrite', 'ABCDEF\rXY' },
    { 'backspace-overwrite', 'ABC\8\8Z' },
    { 'protected-erase', CSI .. '1"q' .. 'PP' .. CSI .. '0"q' .. 'QQ' .. CSI .. '?2J' },
    { 'lineinfo-dwl', ESC .. '#6' .. 'ABC' },
    { 'lineinfo-dhl', ESC .. '#3' .. 'ABC' .. '\r\n' .. ESC .. '#4' .. 'DEF' },
    { 'nothing', '' },
  }
  for merge = 0, 3 do
    for _, s in ipairs(SCRIPTS) do
      for _, want in ipairs({ false, true }) do
        local t = newterm({ mode = 'screen', damage = merge, altscreen = true, want_moverect = want })
        t:write(s[2])
        screen_answer(
          string.format('v9/m%d/%s/%s', merge, s[1], want and 'mv' or 'nomv'),
          t,
          'merge=' .. merge
        )
      end
    end
  end
end)

-- ============================================================== v10 resize

section('v10-resize', function()
  local CONTENT = 'row1\r\nrow2\r\nrow3\r\nrow4\r\nrow5\r\nrow6'
  local SIZES = {
    { 6, 20 },
    { 3, 20 },
    { 12, 20 },
    { 6, 10 },
    { 6, 40 },
    { 1, 1 },
    { 2, 3 },
    { 6, 21 },
    { 7, 19 },
    { 24, 80 },
  }
  for _, reflow in ipairs({ false, true }) do
    for _, size in ipairs(SIZES) do
      local t = newterm({ mode = 'screen', rows = 6, cols = 20, reflow = reflow })
      t:write(CONTENT)
      t:reset()
      C.vterm_set_size(t.vt, size[1], size[2])
      t.rows, t.cols = size[1], size[2]
      screen_answer(
        string.format('v10/resize/%s/%dx%d', reflow and 'reflow' or 'noreflow', size[1], size[2]),
        t,
        string.format('sb=%d', #t.rec.sb)
      )
    end
  end
  -- Long wrapped content is what reflow actually rewrites.
  for _, reflow in ipairs({ false, true }) do
    for _, cols in ipairs({ 5, 10, 15, 19, 21, 40 }) do
      local t = newterm({ mode = 'screen', rows = 6, cols = 20, reflow = reflow })
      t:write(string.rep('abcdefghij', 6))
      t:reset()
      C.vterm_set_size(t.vt, 6, cols)
      t.cols = cols
      screen_answer(
        string.format('v10/reflow/%s/c%d', reflow and 'on' or 'off', cols),
        t,
        string.format('sb=%d', #t.rec.sb)
      )
    end
  end
  -- Scrollback: push on scroll, pop on grow.
  for _, n in ipairs({ 1, 2, 6, 7, 13, 30 }) do
    local t = newterm({ mode = 'screen', rows = 6, cols = 20 })
    t:write(string.rep('L\r\n', n))
    local pushed = #t.rec.sb
    t:reset()
    C.vterm_set_size(t.vt, 12, 20)
    t.rows = 12
    screen_answer('v10/sb/grow/' .. n, t, string.format('pushed=%d left=%d', pushed, #t.rec.sb))
  end
  for _, n in ipairs({ 1, 6, 13 }) do
    local t = newterm({ mode = 'screen', rows = 6, cols = 20 })
    t:write(string.rep('L\r\n', n))
    t:reset()
    C.vterm_set_size(t.vt, 3, 20)
    t.rows = 3
    screen_answer('v10/sb/shrink/' .. n, t, string.format('left=%d', #t.rec.sb))
  end
  -- The altscreen has no scrollback, and swapping must not lose the primary.
  for _, seq in ipairs({ '?47h', '?47l', '?1047h', '?1047l', '?1048h', '?1048l', '?1049h', '?1049l' }) do
    for _, en in ipairs({ false, true }) do
      local t = newterm({ mode = 'screen', altscreen = en })
      t:write('PRIMARY\r\nSECOND')
      t:reset()
      t:write(CSI .. seq .. 'ALT')
      screen_answer('v10/alt/' .. seq:gsub('%?', 'q') .. '/' .. (en and 'en' or 'dis'), t)
    end
  end
  -- Reset, hard and soft, from a dirtied terminal.
  for _, hard in ipairs({ 0, 1 }) do
    local t = newterm({ mode = 'screen', altscreen = true })
    t:write(CSI .. '1;31m' .. 'DIRTY' .. CSI .. '2;5r' .. CSI .. '?6h' .. CSI .. '?1049h')
    t:reset()
    C.vterm_screen_reset(t.screen, hard)
    screen_answer('v10/reset/screen' .. hard, t)
  end
  for _, hard in ipairs({ 0, 1 }) do
    local t = newterm({ mode = 'state' })
    t:write(CSI .. '1;31m' .. 'DIRTY' .. CSI .. '2;5r' .. CSI .. '?6h')
    t:reset()
    C.vterm_state_reset(t.state, hard)
    state_answer('v10/reset/state' .. hard, t)
  end
end)

-- =========================================================== v91 abortprobe

local PROBES = {
  {
    'huge-size',
    function()
      local t = newterm({ mode = 'screen' })
      C.vterm_set_size(t.vt, 2000, 2000)
      t.rows, t.cols = 2000, 2000
      return t:write('x')
    end,
  },
  {
    'zero-size',
    function()
      local t = newterm({ mode = 'screen' })
      C.vterm_set_size(t.vt, 0, 0)
      return t:write('x')
    end,
  },
  {
    'negative-size',
    function()
      local t = newterm({ mode = 'screen' })
      C.vterm_set_size(t.vt, -1, -1)
      return t:write('x')
    end,
  },
  {
    'one-by-one',
    function()
      local t = newterm({ mode = 'screen', rows = 1, cols = 1 })
      return t:write(string.rep('\228\184\173', 20))
    end,
  },
  {
    'csi-arg-overflow',
    function()
      local t = newterm({ mode = 'screen' })
      return t:write(CSI .. string.rep('9', 40) .. 'H')
    end,
  },
  {
    'csi-many-args',
    function()
      local t = newterm({ mode = 'screen' })
      return t:write(CSI .. string.rep('1;', 200) .. '1H')
    end,
  },
  {
    'csi-many-intermed',
    function()
      local t = newterm({ mode = 'screen' })
      return t:write(CSI .. string.rep(' ', 200) .. 'q')
    end,
  },
  {
    'csi-many-leader',
    function()
      local t = newterm({ mode = 'screen' })
      return t:write(CSI .. string.rep('?', 200) .. '1h')
    end,
  },
  {
    'esc-many-intermed',
    function()
      local t = newterm({ mode = 'screen' })
      return t:write(ESC .. string.rep('#', 200) .. '8')
    end,
  },
  {
    'osc-huge',
    function()
      local t = newterm({ mode = 'screen' })
      return t:write(ESC .. ']0;' .. string.rep('a', 200000) .. ST)
    end,
  },
  {
    'dcs-huge',
    function()
      local t = newterm({ mode = 'screen' })
      return t:write(ESC .. 'P' .. string.rep('b', 200000) .. ST)
    end,
  },
  {
    'sel-huge',
    function()
      local t = newterm({ mode = 'screen', selection = true, selbuflen = 8 })
      return t:write(ESC .. ']52;c;' .. string.rep('YQ==', 20000) .. ST)
    end,
  },
  {
    'sel-nobuffer',
    function()
      local t = newterm({ mode = 'state', selection = true, selbuflen = 0 })
      return t:write(ESC .. ']52;c;aGVsbG8=' .. ST)
    end,
  },
  {
    'scroll-huge',
    function()
      local t = newterm({ mode = 'screen' })
      return t:write(CSI .. '2147483647S')
    end,
  },
  {
    'scroll-region-inverted',
    function()
      local t = newterm({ mode = 'screen' })
      t:write(CSI .. '6;1r')
      return t:write(string.rep('x\r\n', 20))
    end,
  },
  {
    'margins-inverted',
    function()
      local t = newterm({ mode = 'screen' })
      t:write(CSI .. '?69h' .. CSI .. '15;2s')
      return t:write(string.rep('x', 40))
    end,
  },
  {
    'insert-huge',
    function()
      local t = newterm({ mode = 'screen' })
      return t:write('abc' .. CSI .. '2147483647@')
    end,
  },
  {
    'delete-huge',
    function()
      local t = newterm({ mode = 'screen' })
      return t:write('abc' .. CSI .. '2147483647P')
    end,
  },
  {
    'erase-huge',
    function()
      local t = newterm({ mode = 'screen' })
      return t:write('abc' .. CSI .. '2147483647X')
    end,
  },
  {
    'repeat-huge',
    function()
      local t = newterm({ mode = 'screen' })
      return t:write('a' .. CSI .. '2147483647b')
    end,
  },
  {
    'tab-huge',
    function()
      local t = newterm({ mode = 'screen' })
      return t:write(CSI .. '2147483647I' .. CSI .. '2147483647Z')
    end,
  },
  {
    'kitty-overflow',
    function()
      local t = newterm({ mode = 'state' })
      for _ = 1, 200 do
        t:write(CSI .. '>31u')
      end
      return t:write(CSI .. '<200u')
    end,
  },
  {
    'palette-out-of-range',
    function()
      local t = newterm({ mode = 'screen' })
      return t:write(ESC .. ']4;99999;#ff0000' .. ST)
    end,
  },
  {
    'combining-flood',
    function()
      local t = newterm({ mode = 'screen' })
      return t:write('a' .. string.rep('\204\129', 500))
    end,
  },
  {
    'grapheme-buf-overflow',
    function()
      local t = newterm({ mode = 'screen' })
      return t:write(string.rep('\240\159\145\168\226\128\141', 40) .. '\240\159\146\187')
    end,
  },
  {
    'wide-at-last-col',
    function()
      local t = newterm({ mode = 'screen', rows = 2, cols = 2 })
      return t:write('x\228\184\173')
    end,
  },
  {
    'reflow-to-one-col',
    function()
      local t = newterm({ mode = 'screen', reflow = true })
      t:write(string.rep('abcdefghij', 6))
      C.vterm_set_size(t.vt, 6, 1)
      return t:write('z')
    end,
  },
  {
    'sb-pop-empty',
    function()
      local t = newterm({ mode = 'screen', rows = 3, cols = 8 })
      C.vterm_set_size(t.vt, 20, 8)
      t.rows = 20
      return t:write('z')
    end,
  },
  {
    'mouse-negative',
    function()
      local t = newterm({ mode = 'state' })
      t:write(CSI .. '?1003h' .. CSI .. '?1006h')
      C.vterm_mouse_move(t.vt, -5, -5, 0)
      C.vterm_mouse_button(t.vt, -1, true, 0)
      C.vterm_mouse_button(t.vt, 99, true, 0)
      return 1
    end,
  },
  {
    'key-past-max',
    function()
      local t = newterm({ mode = 'state' })
      C.vterm_keyboard_key(t.vt, 4294967295, 7)
      C.vterm_keyboard_unichar(t.vt, 4294967295, 7)
      return 1
    end,
  },
  {
    'output-overflow',
    function()
      local t = newterm({ mode = 'state' })
      C.vterm_output_set_callback(t.vt, nil, nil)
      for _ = 1, 5000 do
        t:write(CSI .. '5n')
      end
      return 1
    end,
  },
  {
    'no-callbacks',
    function()
      local vt = C.vterm_new(6, 20)
      C.vterm_set_utf8(vt, 1)
      local st = C.vterm_obtain_state(vt)
      C.vterm_state_reset(st, 1)
      local s = 'abc' .. CSI .. '2J' .. ESC .. ']0;t' .. ST
      return tonumber(C.vterm_input_write(vt, s, #s))
    end,
  },
}

if probe then
  for _, p in ipairs(PROBES) do
    if p[1] == probe then
      local ok, res = pcall(p[2])
      io.write(string.format('PROBE %s ok=%s r=%s\n', probe, tostring(ok), esc(tostring(res))))
      os.exit(0)
    end
  end
  io.write('PROBE ' .. probe .. ' NOTFOUND\n')
  os.exit(3)
end

section('v91-abortprobe', function()
  local nvim = os.getenv('VT_NVIM')
  local script = os.getenv('VT_SCRIPT') or debug.getinfo(1, 'S').source:sub(2)
  local aborted = 0
  for _, p in ipairs(PROBES) do
    local cmd = string.format(
      'VT_PROBE=%s timeout -k 2 60 %s -u NONE -i NONE -l %s 2>&1; echo "EXIT=$?"',
      p[1],
      nvim,
      script
    )
    local fh = io.popen(cmd)
    local text = fh:read('*a') or ''
    fh:close()
    local code = tonumber(text:match('EXIT=(%d+)') or '-1')
    text = text:gsub('EXIT=%d+%s*$', '')
    -- A panicking child prints its own pid ("thread 'main' (4113724)
    -- panicked at ..."), which is the one number in the whole sweep that
    -- moves run to run.
    text = text:gsub('%(%d%d+%)', '(<PID>)')
    -- A panic also names the source line it fired on, which moves whenever
    -- anything above it in the file is edited -- a comment is enough. The
    -- file is the signal; the line and column are churn.
    text = text:gsub('(%.rs):%d+:%d+:', '%1:<LINE>:<COL>:')
    if code ~= 0 then
      aborted = aborted + 1
    end
    answer('v91/' .. p[1], {
      string.format('exit=%d said=%s', code, esc(cap(text:gsub('%s+$', ''), 300))),
    }, { exit = code, said = text })
  end
  emit('v91', 'groups', string.format('cases=%d aborted=%d', #PROBES, aborted))
end)

run_sections()
if structfd then
  structfd:close()
end
