#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "nvim/grid.h"
#include "nvim/mbyte.h"
#include "nvim/vterm/parser.h"
#include "nvim/vterm/pen.h"
#include "nvim/vterm/state.h"
#include "nvim/vterm/vterm.h"
#include "nvim/vterm/vterm_internal_defs.h"

#include "vterm/state.c.generated.h"

// Rust FFI: encoding lookup returning VTermEncoding*
extern VTermEncoding *rs_vterm_lookup_encoding_ptr(int enc_type, char designation);

// Rust FFI declarations (Phase 1)
extern void rs_vterm_state_set_mode(VTermState *state, int num, int val);
extern void rs_vterm_state_set_dec_mode(VTermState *state, int num, int val);
extern void rs_vterm_state_request_dec_mode(VTermState *state, int num);
extern void rs_vterm_state_request_version_string(VTermState *state);
extern void rs_vterm_state_savecursor(VTermState *state, int save);

// Rust FFI declarations (Phase 2)
extern void rs_vterm_state_set_lineinfo(VTermState *state, int row, int force, int dwl, int dhl);
extern int rs_vterm_state_on_escape(VTermState *state, const char *bytes, size_t len);
extern void rs_vterm_state_request_key_encoding_flags(VTermState *state);
extern void rs_vterm_state_set_key_encoding_flags(VTermState *state, int arg, int mode);
extern void rs_vterm_state_push_key_encoding_flags(VTermState *state, int arg);
extern void rs_vterm_state_pop_key_encoding_flags(VTermState *state, int arg);

// Rust FFI declarations (Phase 3)
extern int rs_vterm_state_on_control(VTermState *state, uint8_t control);

// Rust FFI declarations (Phase 4)
extern int rs_vterm_state_on_csi(VTermState *state, const char *leader, const long args[],
                                  int argcount, const char *intermed, char command);

// Rust FFI declarations (Phase 5)
extern int rs_vterm_state_on_osc(VTermState *state, int command, VTermStringFragment frag);
extern int rs_vterm_state_on_dcs(VTermState *state, const char *command, size_t commandlen,
                                  VTermStringFragment frag);
extern int rs_vterm_state_on_apc(VTermState *state, VTermStringFragment frag);
extern int rs_vterm_state_on_pm(VTermState *state, VTermStringFragment frag);
extern int rs_vterm_state_on_sos(VTermState *state, VTermStringFragment frag);

// Rust FFI declarations (Phase 6)
extern int rs_vterm_state_set_termprop(VTermState *state, int prop, VTermValue *val);
extern int rs_vterm_state_on_text(VTermState *state, const char bytes[], size_t len);
extern int rs_vterm_state_on_resize(VTermState *state, int rows, int cols);
extern void rs_vterm_state_reset(VTermState *state, int hard);
extern void rs_vterm_state_set_unrecognised_fallbacks(VTermState *state,
                                                       const VTermStateFallbacks *fallbacks,
                                                       void *user);
extern void rs_vterm_state_focus_in(VTermState *state);
extern void rs_vterm_state_focus_out(VTermState *state);
extern void rs_vterm_state_set_selection_callbacks(VTermState *state,
                                                    const VTermSelectionCallbacks *callbacks,
                                                    void *user,
                                                    char *buffer,
                                                    size_t buflen);

#define strneq(a, b, n) (strncmp(a, b, n) == 0)

#define LBOUND(v, min) if ((v) < (min))(v) = (min)
#define UBOUND(v, max) if ((v) > (max))(v) = (max)

// Primary Device Attributes (DA1) response.
// We make this a global (extern) variable so that we can override it with FFI
// in tests.
char vterm_primary_device_attr[] = "61;22;52";

// Some convenient wrappers to make callback functions easier

static void putglyph(VTermState *state, const schar_T schar, int width, VTermPos pos)
{
  VTermGlyphInfo info = {
    .schar = schar,
    .width = width,
    .protected_cell = state->protected_cell,
    .dwl = state->lineinfo[pos.row].doublewidth,
    .dhl = state->lineinfo[pos.row].doubleheight,
  };

  if (state->callbacks && state->callbacks->putglyph) {
    if ((*state->callbacks->putglyph)(&info, pos, state->cbdata)) {
      return;
    }
  }

  DEBUG_LOG("libvterm: Unhandled putglyph U+%04x at (%d,%d)\n", chars[0], pos.col, pos.row);
}

static VTermState *vterm_state_new(VTerm *vt)
{
  VTermState *state = vterm_allocator_malloc(vt, sizeof(VTermState));

  state->vt = vt;

  state->rows = vt->rows;
  state->cols = vt->cols;

  state->mouse_col = 0;
  state->mouse_row = 0;
  state->mouse_buttons = 0;

  state->mouse_protocol = MOUSE_X10;

  state->callbacks = NULL;
  state->cbdata = NULL;

  state->selection.callbacks = NULL;
  state->selection.user = NULL;
  state->selection.buffer = NULL;

  vterm_state_newpen(state);

  state->bold_is_highbright = 0;

  state->combine_pos.row = -1;

  state->tabstops = vterm_allocator_malloc(state->vt, ((size_t)state->cols + 7) / 8);

  state->lineinfos[BUFIDX_PRIMARY] = vterm_allocator_malloc(state->vt,
                                                            (size_t)state->rows *
                                                            sizeof(VTermLineInfo));
  // TODO(vterm): Make an 'enable' function
  state->lineinfos[BUFIDX_ALTSCREEN] = vterm_allocator_malloc(state->vt,
                                                              (size_t)state->rows *
                                                              sizeof(VTermLineInfo));
  state->lineinfo = state->lineinfos[BUFIDX_PRIMARY];

  state->encoding_utf8.enc = rs_vterm_lookup_encoding_ptr(ENC_UTF8, 'u');
  if (state->encoding_utf8.enc && state->encoding_utf8.enc->init) {
    (*state->encoding_utf8.enc->init)(state->encoding_utf8.enc, state->encoding_utf8.data);
  }

  for (size_t i = 0; i < ARRAY_SIZE(state->key_encoding_stacks); i++) {
    struct VTermKeyEncodingStack *stack = &state->key_encoding_stacks[i];
    for (size_t j = 0; j < ARRAY_SIZE(stack->items); j++) {
      memset(&stack->items[j], 0, sizeof(stack->items[j]));
    }

    stack->size = 1;
  }

  return state;
}

void vterm_state_free(VTermState *state)
{
  vterm_allocator_free(state->vt, state->tabstops);
  vterm_allocator_free(state->vt, state->lineinfos[BUFIDX_PRIMARY]);
  if (state->lineinfos[BUFIDX_ALTSCREEN]) {
    vterm_allocator_free(state->vt, state->lineinfos[BUFIDX_ALTSCREEN]);
  }
  vterm_allocator_free(state->vt, state);
}

static int on_text(const char bytes[], size_t len, void *user)
{
  VTermState *state = user;
  return rs_vterm_state_on_text(state, bytes, len);
}

static int on_control(uint8_t control, void *user)
{
  VTermState *state = user;
  return rs_vterm_state_on_control(state, control);
}


static int on_escape(const char *bytes, size_t len, void *user)
{
  VTermState *state = user;
  return rs_vterm_state_on_escape(state, bytes, len);
}

static void set_mode(VTermState *state, int num, int val)
{
  rs_vterm_state_set_mode(state, num, val);
}

static void set_dec_mode(VTermState *state, int num, int val)
{
  rs_vterm_state_set_dec_mode(state, num, val);
}

static void request_dec_mode(VTermState *state, int num)
{
  rs_vterm_state_request_dec_mode(state, num);
}

static void request_version_string(VTermState *state)
{
  rs_vterm_state_request_version_string(state);
}

static void request_key_encoding_flags(VTermState *state)
{
  rs_vterm_state_request_key_encoding_flags(state);
}

static void set_key_encoding_flags(VTermState *state, int arg, int mode)
{
  rs_vterm_state_set_key_encoding_flags(state, arg, mode);
}

static void push_key_encoding_flags(VTermState *state, int arg)
{
  rs_vterm_state_push_key_encoding_flags(state, arg);
}

static void pop_key_encoding_flags(VTermState *state, int arg)
{
  rs_vterm_state_pop_key_encoding_flags(state, arg);
}

static int on_csi(const char *leader, const long args[], int argcount, const char *intermed,
                  char command, void *user)
{
  VTermState *state = user;
  return rs_vterm_state_on_csi(state, leader, args, argcount, intermed, command);
}

static uint8_t unbase64one(char c)
{
  if (c >= 'A' && c <= 'Z') {
    return (uint8_t)c - 'A';
  } else if (c >= 'a' && c <= 'z') {
    return (uint8_t)c - 'a' + 26;
  } else if (c >= '0' && c <= '9') {
    return (uint8_t)c - '0' + 52;
  } else if (c == '+') {
    return 62;
  } else if (c == '/') {
    return 63;
  }

  return 0xFF;
}

static void osc_selection(VTermState *state, VTermStringFragment frag)
{
  if (frag.initial) {
    state->tmp.selection.mask = 0;
    state->tmp.selection.state = SELECTION_INITIAL;
  }

  while (!state->tmp.selection.state && frag.len) {
    // Parse selection parameter
    switch (frag.str[0]) {
    case 'c':
      state->tmp.selection.mask |= VTERM_SELECTION_CLIPBOARD;
      break;
    case 'p':
      state->tmp.selection.mask |= VTERM_SELECTION_PRIMARY;
      break;
    case 'q':
      state->tmp.selection.mask |= VTERM_SELECTION_SECONDARY;
      break;
    case 's':
      state->tmp.selection.mask |= VTERM_SELECTION_SELECT;
      break;
    case '0':
    case '1':
    case '2':
    case '3':
    case '4':
    case '5':
    case '6':
    case '7':
      state->tmp.selection.mask |= (VTERM_SELECTION_CUT0 << (frag.str[0] - '0'));
      break;

    case ';':
      state->tmp.selection.state = SELECTION_SELECTED;
      if (!state->tmp.selection.mask) {
        state->tmp.selection.mask = VTERM_SELECTION_SELECT|VTERM_SELECTION_CUT0;
      }
      break;
    }

    frag.str++;
    frag.len--;
  }

  if (!frag.len) {
    // Clear selection if we're already finished but didn't do anything
    if (frag.final && state->selection.callbacks->set) {
      (*state->selection.callbacks->set)(state->tmp.selection.mask, (VTermStringFragment){
        .str = NULL,
        .len = 0,
        .initial = state->tmp.selection.state != SELECTION_SET,
        .final = true,
      }, state->selection.user);
    }
    return;
  }

  if (state->tmp.selection.state == SELECTION_SELECTED) {
    if (frag.str[0] == '?') {
      state->tmp.selection.state = SELECTION_QUERY;
    } else {
      state->tmp.selection.state = SELECTION_SET_INITIAL;
      state->tmp.selection.recvpartial = 0;
    }
  }

  if (state->tmp.selection.state == SELECTION_QUERY) {
    if (state->selection.callbacks->query) {
      (*state->selection.callbacks->query)(state->tmp.selection.mask, state->selection.user);
    }
    return;
  }

  if (state->tmp.selection.state == SELECTION_INVALID) {
    return;
  }

  if (state->selection.callbacks->set) {
    size_t bufcur = 0;
    char *buffer = state->selection.buffer;

    uint32_t x = 0;  // Current decoding value
    int n = 0;      // Number of sextets consumed

    if (state->tmp.selection.recvpartial) {
      n = state->tmp.selection.recvpartial >> 24;
      x = state->tmp.selection.recvpartial & 0x03FFFF;  // could be up to 18 bits of state in here

      state->tmp.selection.recvpartial = 0;
    }

    while ((state->selection.buflen - bufcur) >= 3 && frag.len) {
      if (frag.str[0] == '=') {
        if (n == 2) {
          buffer[0] = (char)(x >> 4 & 0xFF);
          buffer += 1, bufcur += 1;
        }
        if (n == 3) {
          buffer[0] = (char)(x >> 10 & 0xFF);
          buffer[1] = (char)(x >>  2 & 0xFF);
          buffer += 2, bufcur += 2;
        }

        while (frag.len && frag.str[0] == '=') {
          frag.str++, frag.len--;
        }

        n = 0;
      } else {
        uint8_t b = unbase64one(frag.str[0]);
        if (b == 0xFF) {
          DEBUG_LOG("base64decode bad input %02X\n", (uint8_t)frag.str[0]);

          state->tmp.selection.state = SELECTION_INVALID;
          if (state->selection.callbacks->set) {
            (*state->selection.callbacks->set)(state->tmp.selection.mask, (VTermStringFragment){
              .str = NULL,
              .len = 0,
              .initial = true,
              .final = true,
            }, state->selection.user);
          }
          break;
        }

        x = (x << 6) | b;
        n++;
        frag.str++, frag.len--;

        if (n == 4) {
          buffer[0] = (char)(x >> 16 & 0xFF);
          buffer[1] = (char)(x >>  8 & 0xFF);
          buffer[2] = (char)(x >>  0 & 0xFF);

          buffer += 3, bufcur += 3;
          x = 0;
          n = 0;
        }
      }

      if (!frag.len || (state->selection.buflen - bufcur) < 3) {
        if (bufcur) {
          (*state->selection.callbacks->set)(state->tmp.selection.mask, (VTermStringFragment){
            .str = state->selection.buffer,
            .len = bufcur,
            .initial = state->tmp.selection.state == SELECTION_SET_INITIAL,
            .final = frag.final && !frag.len,
          }, state->selection.user);
          state->tmp.selection.state = SELECTION_SET;
        }

        buffer = state->selection.buffer;
        bufcur = 0;
      }
    }

    if (n) {
      state->tmp.selection.recvpartial = (uint32_t)(n << 24) | x;
    }
  }
}

static int on_osc(int command, VTermStringFragment frag, void *user)
{
  VTermState *state = user;
  return rs_vterm_state_on_osc(state, command, frag);
}

static void request_status_string(VTermState *state, VTermStringFragment frag)
{
  VTerm *vt = state->vt;

  char *tmp = state->tmp.decrqss;

  if (frag.initial) {
    tmp[0] = tmp[1] = tmp[2] = tmp[3] = 0;
  }

  size_t i = 0;
  while (i < sizeof(state->tmp.decrqss) - 1 && tmp[i]) {
    i++;
  }
  while (i < sizeof(state->tmp.decrqss) - 1 && frag.len--) {
    tmp[i++] = (frag.str++)[0];
  }
  tmp[i] = 0;

  if (!frag.final) {
    return;
  }

  switch (tmp[0] | tmp[1] << 8 | tmp[2] << 16) {
  case 'm': {
    // Query SGR
    long args[20];
    int argc = vterm_state_getpen(state, args, sizeof(args)/sizeof(args[0]));
    size_t cur = 0;

    cur += (size_t)snprintf(vt->tmpbuffer + cur, vt->tmpbuffer_len - cur,
                            vt->mode.ctrl8bit ? "\x90" "1$r" : ESC_S "P" "1$r");  // DCS 1$r ...
    if (cur >= vt->tmpbuffer_len) {
      return;
    }

    for (int argi = 0; argi < argc; argi++) {
      cur += (size_t)snprintf(vt->tmpbuffer + cur, vt->tmpbuffer_len - cur,
                              argi == argc - 1 ? "%ld"
                                               : CSI_ARG_HAS_MORE(args[argi]) ? "%ld:"
                                                                              : "%ld;",
                              CSI_ARG(args[argi]));
      if (cur >= vt->tmpbuffer_len) {
        return;
      }
    }

    cur += (size_t)snprintf(vt->tmpbuffer + cur, vt->tmpbuffer_len - cur,
                            vt->mode.ctrl8bit ? "m" "\x9C" : "m" ESC_S "\\");  // ... m ST
    if (cur >= vt->tmpbuffer_len) {
      return;
    }

    vterm_push_output_bytes(vt, vt->tmpbuffer, cur);
    return;
  }

  case 'r':
    // Query DECSTBM
    vterm_push_output_sprintf_str(vt, C1_DCS, true,
                                  "1$r%d;%dr", state->scrollregion_top + 1,
                                  SCROLLREGION_BOTTOM(state));
    return;

  case 's':
    // Query DECSLRM
    vterm_push_output_sprintf_str(vt, C1_DCS, true,
                                  "1$r%d;%ds", SCROLLREGION_LEFT(state) + 1,
                                  SCROLLREGION_RIGHT(state));
    return;

  case ' '|('q' << 8): {
    // Query DECSCUSR
    int reply = 0;
    switch (state->mode.cursor_shape) {
    case VTERM_PROP_CURSORSHAPE_BLOCK:
      reply = 2; break;
    case VTERM_PROP_CURSORSHAPE_UNDERLINE:
      reply = 4; break;
    case VTERM_PROP_CURSORSHAPE_BAR_LEFT:
      reply = 6; break;
    }
    if (state->mode.cursor_blink) {
      reply--;
    }
    vterm_push_output_sprintf_str(vt, C1_DCS, true,
                                  "1$r%d q", reply);
    return;
  }

  case '\"'|('q' << 8):
    // Query DECSCA
    vterm_push_output_sprintf_str(vt, C1_DCS, true,
                                  "1$r%d\"q", state->protected_cell ? 1 : 2);
    return;
  }

  vterm_push_output_sprintf_str(state->vt, C1_DCS, true, "0$r");
}

void nvim_vterm_state_request_status_string(VTermState *state, VTermStringFragment frag)
{
  request_status_string(state, frag);
}

void nvim_vterm_state_osc_selection(VTermState *state, VTermStringFragment frag)
{
  osc_selection(state, frag);
}

static int on_dcs(const char *command, size_t commandlen, VTermStringFragment frag, void *user)
{
  VTermState *state = user;
  return rs_vterm_state_on_dcs(state, command, commandlen, frag);
}

static int on_apc(VTermStringFragment frag, void *user)
{
  VTermState *state = user;
  return rs_vterm_state_on_apc(state, frag);
}

static int on_pm(VTermStringFragment frag, void *user)
{
  VTermState *state = user;
  return rs_vterm_state_on_pm(state, frag);
}

static int on_sos(VTermStringFragment frag, void *user)
{
  VTermState *state = user;
  return rs_vterm_state_on_sos(state, frag);
}

static int on_resize(int rows, int cols, void *user)
{
  VTermState *state = user;
  return rs_vterm_state_on_resize(state, rows, cols);
}

static const VTermParserCallbacks parser_callbacks = {
  .text = on_text,
  .control = on_control,
  .escape = on_escape,
  .csi = on_csi,
  .osc = on_osc,
  .dcs = on_dcs,
  .apc = on_apc,
  .pm = on_pm,
  .sos = on_sos,
  .resize = on_resize,
};

VTermState *vterm_obtain_state(VTerm *vt)
{
  if (vt->state) {
    return vt->state;
  }

  VTermState *state = vterm_state_new(vt);
  vt->state = state;

  vterm_parser_set_callbacks(vt, &parser_callbacks, state);

  return state;
}

void vterm_state_reset(VTermState *state, int hard)
{
  rs_vterm_state_reset(state, hard);
}

void vterm_state_set_callbacks(VTermState *state, const VTermStateCallbacks *callbacks, void *user)
{
  if (callbacks) {
    state->callbacks = callbacks;
    state->cbdata = user;
    if (state->callbacks && state->callbacks->initpen) {
      (*state->callbacks->initpen)(state->cbdata);
    }
  } else {
    state->callbacks = NULL;
    state->cbdata = NULL;
  }
}

void vterm_state_set_unrecognised_fallbacks(VTermState *state, const VTermStateFallbacks *fallbacks,
                                            void *user)
{
  rs_vterm_state_set_unrecognised_fallbacks(state, fallbacks, user);
}

int vterm_state_set_termprop(VTermState *state, VTermProp prop, VTermValue *val)
{
  return rs_vterm_state_set_termprop(state, (int)prop, val);
}

void vterm_state_focus_in(VTermState *state)
{
  rs_vterm_state_focus_in(state);
}

void vterm_state_focus_out(VTermState *state)
{
  rs_vterm_state_focus_out(state);
}

const VTermLineInfo *vterm_state_get_lineinfo(const VTermState *state, int row)
{
  return state->lineinfo + row;
}

void vterm_state_set_selection_callbacks(VTermState *state,
                                         const VTermSelectionCallbacks *callbacks, void *user,
                                         char *buffer, size_t buflen)
{
  rs_vterm_state_set_selection_callbacks(state, callbacks, user, buffer, buflen);
}

// C Accessor Functions for Rust FFI
// These functions provide Rust code access to VTermState fields via the
// opaque handle pattern. Each accessor follows the naming convention:
// nvim_vterm_state_get_<field> / nvim_vterm_state_set_<field>

// --- Dimension accessors ---

int nvim_vterm_state_get_rows(const VTermState *state) { return state->rows; }

int nvim_vterm_state_get_cols(const VTermState *state) { return state->cols; }

// --- Cursor position accessors ---

VTermPos nvim_vterm_state_get_pos(const VTermState *state) { return state->pos; }

void nvim_vterm_state_set_pos(VTermState *state, VTermPos pos) { state->pos = pos; }

int nvim_vterm_state_get_at_phantom(const VTermState *state) { return state->at_phantom; }

void nvim_vterm_state_set_at_phantom(VTermState *state, int at_phantom) { state->at_phantom = at_phantom; }

// --- Scroll region accessors ---

int nvim_vterm_state_get_scrollregion_top(const VTermState *state) { return state->scrollregion_top; }

void nvim_vterm_state_set_scrollregion_top(VTermState *state, int top) { state->scrollregion_top = top; }

int nvim_vterm_state_get_scrollregion_bottom(const VTermState *state) { return state->scrollregion_bottom; }

void nvim_vterm_state_set_scrollregion_bottom(VTermState *state, int bottom) { state->scrollregion_bottom = bottom; }

int nvim_vterm_state_get_scrollregion_left(const VTermState *state) { return state->scrollregion_left; }

void nvim_vterm_state_set_scrollregion_left(VTermState *state, int left) { state->scrollregion_left = left; }

int nvim_vterm_state_get_scrollregion_right(const VTermState *state) { return state->scrollregion_right; }

void nvim_vterm_state_set_scrollregion_right(VTermState *state, int right) { state->scrollregion_right = right; }

// Computed scroll region bounds (using macros from the header)
int nvim_vterm_state_scrollregion_bottom(const VTermState *state) { return SCROLLREGION_BOTTOM(state); }

int nvim_vterm_state_scrollregion_left(const VTermState *state) { return SCROLLREGION_LEFT(state); }

int nvim_vterm_state_scrollregion_right(const VTermState *state) { return SCROLLREGION_RIGHT(state); }

int nvim_vterm_state_row_width(const VTermState *state, int row) { return ROWWIDTH(state, row); }

int nvim_vterm_state_this_row_width(const VTermState *state) { return THISROWWIDTH(state); }

// --- Line info accessors ---

VTermLineInfo *nvim_vterm_state_get_lineinfo(VTermState *state) { return state->lineinfo; }

VTermLineInfo *nvim_vterm_state_get_lineinfo_at(VTermState *state, int row) { return &state->lineinfo[row]; }

void nvim_vterm_state_set_lineinfo_continuation(VTermState *state, int row, int continuation)
{
  state->lineinfo[row].continuation = (unsigned)continuation;
}

// --- Mode accessors ---

int nvim_vterm_state_mode_autowrap(const VTermState *state) { return state->mode.autowrap; }

int nvim_vterm_state_mode_insert(const VTermState *state) { return state->mode.insert; }

int nvim_vterm_state_mode_newline(const VTermState *state) { return state->mode.newline; }

int nvim_vterm_state_mode_origin(const VTermState *state) { return state->mode.origin; }

int nvim_vterm_state_mode_cursor_visible(const VTermState *state) { return state->mode.cursor_visible; }

int nvim_vterm_state_mode_leftrightmargin(const VTermState *state) { return state->mode.leftrightmargin; }

int nvim_vterm_state_mode_alt_screen(const VTermState *state) { return state->mode.alt_screen; }

// --- Protected cell accessor ---

int nvim_vterm_state_get_protected_cell(const VTermState *state) { return state->protected_cell; }

// --- Callback accessors ---

const VTermStateCallbacks *nvim_vterm_state_get_callbacks(const VTermState *state) { return state->callbacks; }

void *nvim_vterm_state_get_cbdata(const VTermState *state) { return state->cbdata; }

// --- VTerm handle accessor ---

VTerm *nvim_vterm_state_get_vt(const VTermState *state) { return state->vt; }

// --- Grapheme buffer accessors ---

char *nvim_vterm_state_get_grapheme_buf(VTermState *state) { return state->grapheme_buf; }

size_t nvim_vterm_state_get_grapheme_len(const VTermState *state) { return state->grapheme_len; }

void nvim_vterm_state_set_grapheme_len(VTermState *state, size_t len) { state->grapheme_len = len; }

int nvim_vterm_state_get_combine_width(const VTermState *state) { return state->combine_width; }

void nvim_vterm_state_set_combine_width(VTermState *state, int width) { state->combine_width = width; }

VTermPos nvim_vterm_state_get_combine_pos(const VTermState *state) { return state->combine_pos; }

void nvim_vterm_state_set_combine_pos(VTermState *state, VTermPos pos) { state->combine_pos = pos; }

// --- Lineinfo scroll helpers ---

void nvim_vterm_state_lineinfo_scroll_down(VTermState *state, int start_row, int end_row,
                                           int count)
{
  // Scroll lineinfo down by `count` rows (shift entries up in memory)
  int height = end_row - start_row - count;
  if (height > 0) {
    memmove(state->lineinfo + start_row,
            state->lineinfo + start_row + count,
            (size_t)height * sizeof(state->lineinfo[0]));
  }
  // Clear the new rows at the bottom
  for (int row = end_row - count; row < end_row; row++) {
    state->lineinfo[row] = (VTermLineInfo){ 0 };
  }
}

void nvim_vterm_state_lineinfo_scroll_up(VTermState *state, int start_row, int end_row, int count)
{
  // Scroll lineinfo up by `count` rows (shift entries down in memory)
  int height = end_row - start_row - count;
  if (height > 0) {
    memmove(state->lineinfo + start_row + count,
            state->lineinfo + start_row,
            (size_t)height * sizeof(state->lineinfo[0]));
  }
  // Clear the new rows at the top
  for (int row = start_row; row < start_row + count; row++) {
    state->lineinfo[row] = (VTermLineInfo){ 0 };
  }
}

void nvim_vterm_state_lineinfo_clear(VTermState *state, int row)
{
  state->lineinfo[row] = (VTermLineInfo){ 0 };
}


int nvim_vterm_state_is_col_tabstop(const VTermState *state, int col)
{
  uint8_t mask = (uint8_t)(1 << (col & 7));
  return state->tabstops[col >> 3] & mask;
}

void nvim_vterm_state_set_col_tabstop(VTermState *state, int col)
{
  uint8_t mask = (uint8_t)(1 << (col & 7));
  state->tabstops[col >> 3] |= mask;
}

void nvim_vterm_state_clear_col_tabstop(VTermState *state, int col)
{
  uint8_t mask = (uint8_t)(1 << (col & 7));
  state->tabstops[col >> 3] &= ~mask;
}

// --- VTerm scroll_rect helper ---

void nvim_vterm_scroll_rect(VTermRect rect, int downward, int rightward,
                            int (*moverect)(VTermRect dest, VTermRect src, void *user),
                            int (*erase)(VTermRect rect, int selective, void *user), void *user)
{
  vterm_scroll_rect(rect, downward, rightward, moverect, erase, user);
}

// --- Pen attribute accessors ---

// Scalar pen fields

int nvim_vterm_state_get_pen_bold(const VTermState *state) { return (int)state->pen.bold; }

void nvim_vterm_state_set_pen_bold(VTermState *state, int val) { state->pen.bold = (unsigned)val; }

int nvim_vterm_state_get_pen_underline(const VTermState *state) { return (int)state->pen.underline; }

void nvim_vterm_state_set_pen_underline(VTermState *state, int val) { state->pen.underline = (unsigned)val; }

int nvim_vterm_state_get_pen_italic(const VTermState *state) { return (int)state->pen.italic; }

void nvim_vterm_state_set_pen_italic(VTermState *state, int val) { state->pen.italic = (unsigned)val; }

int nvim_vterm_state_get_pen_blink(const VTermState *state) { return (int)state->pen.blink; }

void nvim_vterm_state_set_pen_blink(VTermState *state, int val) { state->pen.blink = (unsigned)val; }

int nvim_vterm_state_get_pen_reverse(const VTermState *state) { return (int)state->pen.reverse; }

void nvim_vterm_state_set_pen_reverse(VTermState *state, int val) { state->pen.reverse = (unsigned)val; }

int nvim_vterm_state_get_pen_conceal(const VTermState *state) { return (int)state->pen.conceal; }

void nvim_vterm_state_set_pen_conceal(VTermState *state, int val) { state->pen.conceal = (unsigned)val; }

int nvim_vterm_state_get_pen_strike(const VTermState *state) { return (int)state->pen.strike; }

void nvim_vterm_state_set_pen_strike(VTermState *state, int val) { state->pen.strike = (unsigned)val; }

int nvim_vterm_state_get_pen_font(const VTermState *state) { return (int)state->pen.font; }

void nvim_vterm_state_set_pen_font(VTermState *state, int val) { state->pen.font = (unsigned)val; }

int nvim_vterm_state_get_pen_small(const VTermState *state) { return (int)state->pen.small; }

void nvim_vterm_state_set_pen_small(VTermState *state, int val) { state->pen.small = (unsigned)val; }

int nvim_vterm_state_get_pen_baseline(const VTermState *state) { return (int)state->pen.baseline; }

void nvim_vterm_state_set_pen_baseline(VTermState *state, int val) { state->pen.baseline = (unsigned)val; }

int nvim_vterm_state_get_pen_uri(const VTermState *state) { return state->pen.uri; }

void nvim_vterm_state_set_pen_uri(VTermState *state, int val) { state->pen.uri = val; }

// Color pen fields

VTermColor nvim_vterm_state_get_pen_fg(const VTermState *state) { return state->pen.fg; }

void nvim_vterm_state_set_pen_fg(VTermState *state, VTermColor col) { state->pen.fg = col; }

VTermColor nvim_vterm_state_get_pen_bg(const VTermState *state) { return state->pen.bg; }

void nvim_vterm_state_set_pen_bg(VTermState *state, VTermColor col) { state->pen.bg = col; }

// Default fg/bg accessors

VTermColor nvim_vterm_state_get_default_fg(const VTermState *state) { return state->default_fg; }

void nvim_vterm_state_set_default_fg(VTermState *state, VTermColor col) { state->default_fg = col; }

VTermColor nvim_vterm_state_get_default_bg(const VTermState *state) { return state->default_bg; }

void nvim_vterm_state_set_default_bg(VTermState *state, VTermColor col) { state->default_bg = col; }

// Palette color accessors (indices 0-15)

VTermColor nvim_vterm_state_get_color(const VTermState *state, int index) { return state->colors[index]; }

void nvim_vterm_state_set_color(VTermState *state, int index, VTermColor col) { state->colors[index] = col; }

// bold_is_highbright

int nvim_vterm_state_get_bold_is_highbright(const VTermState *state) { return state->bold_is_highbright; }

// Save/restore pen via saved.pen

void nvim_vterm_state_save_pen(VTermState *state) { state->saved.pen = state->pen; }

void nvim_vterm_state_restore_pen(VTermState *state) { state->pen = state->saved.pen; }

// Saved pen field accessors (for reading back after restore)

int nvim_vterm_state_get_saved_pen_bold(const VTermState *state) { return (int)state->saved.pen.bold; }

int nvim_vterm_state_get_saved_pen_underline(const VTermState *state) { return (int)state->saved.pen.underline; }

int nvim_vterm_state_get_saved_pen_italic(const VTermState *state) { return (int)state->saved.pen.italic; }

int nvim_vterm_state_get_saved_pen_blink(const VTermState *state) { return (int)state->saved.pen.blink; }

int nvim_vterm_state_get_saved_pen_reverse(const VTermState *state) { return (int)state->saved.pen.reverse; }

int nvim_vterm_state_get_saved_pen_conceal(const VTermState *state) { return (int)state->saved.pen.conceal; }

int nvim_vterm_state_get_saved_pen_strike(const VTermState *state) { return (int)state->saved.pen.strike; }

int nvim_vterm_state_get_saved_pen_font(const VTermState *state) { return (int)state->saved.pen.font; }

int nvim_vterm_state_get_saved_pen_small(const VTermState *state) { return (int)state->saved.pen.small; }

int nvim_vterm_state_get_saved_pen_baseline(const VTermState *state) { return (int)state->saved.pen.baseline; }

int nvim_vterm_state_get_saved_pen_uri(const VTermState *state) { return state->saved.pen.uri; }

VTermColor nvim_vterm_state_get_saved_pen_fg(const VTermState *state) { return state->saved.pen.fg; }

VTermColor nvim_vterm_state_get_saved_pen_bg(const VTermState *state) { return state->saved.pen.bg; }

// Callback dispatcher for setpenattr

void nvim_vterm_state_call_setpenattr(VTermState *state, int attr, VTermValue *val)
{
  if (state->callbacks && state->callbacks->setpenattr) {
    (*state->callbacks->setpenattr)(attr, val, state->cbdata);
  }
}

// --- Phase 2: VTerm ctrl8bit accessor ---

int nvim_vterm_state_get_vt_ctrl8bit(const VTermState *state) { return state->vt->mode.ctrl8bit; }

void nvim_vterm_state_set_vt_ctrl8bit(VTermState *state, int val) { state->vt->mode.ctrl8bit = (unsigned)val; }

// --- Phase 2: putglyph helper (wraps static putglyph) ---

void nvim_vterm_state_call_putglyph(VTermState *state, schar_T schar, int width, VTermPos pos)
{
  putglyph(state, schar, width, pos);
}

// --- Phase 2: encoding init helper ---

void nvim_vterm_encoding_call_init(void *enc_ptr, void *data)
{
  VTermEncoding *enc = (VTermEncoding *)enc_ptr;
  if (enc && enc->init) {
    (*enc->init)(enc, data);
  }
}

// --- Phase 2: ROWWIDTH helper ---

int nvim_vterm_state_row_width_from_ptr(const VTermState *state, int row)
{
  return ROWWIDTH(state, row);
}

// --- Phase 6: additional accessors ---

// Mode setters
void nvim_vterm_state_set_mode_report_focus(VTermState *state, int val)
{
  state->mode.report_focus = (unsigned)val;
}

void nvim_vterm_state_set_mode_cursor_visible(VTermState *state, int val)
{
  state->mode.cursor_visible = (unsigned)val;
}

void nvim_vterm_state_set_mode_cursor_blink(VTermState *state, int val)
{
  state->mode.cursor_blink = (unsigned)val;
}

void nvim_vterm_state_set_mode_cursor_shape(VTermState *state, int val)
{
  state->mode.cursor_shape = (unsigned)val;
}

void nvim_vterm_state_set_mode_screen(VTermState *state, int val)
{
  state->mode.screen = (unsigned)val;
}

void nvim_vterm_state_set_mode_alt_screen(VTermState *state, int val)
{
  state->mode.alt_screen = (unsigned)val;
}

void nvim_vterm_state_set_mode_theme_updates(VTermState *state, int val)
{
  state->mode.theme_updates = (unsigned)val;
}

void nvim_vterm_state_set_mouse_flags(VTermState *state, int val)
{
  state->mouse_flags = val;
}

int nvim_vterm_state_get_mouse_flags(const VTermState *state)
{
  return state->mouse_flags;
}

// Switch lineinfo to primary or altscreen buffer
void nvim_vterm_state_switch_lineinfo(VTermState *state)
{
  state->lineinfo = state->lineinfos[state->mode.alt_screen ? BUFIDX_ALTSCREEN : BUFIDX_PRIMARY];
}

// Callbacks setter (sets both callbacks ptr and cbdata atomically)
void nvim_vterm_state_set_callbacks_ptr(VTermState *state,
                                         const VTermStateCallbacks *callbacks,
                                         void *user)
{
  state->callbacks = callbacks;
  state->cbdata = user;
}

// Fallbacks setter
void nvim_vterm_state_set_fallbacks_ptr(VTermState *state,
                                         const VTermStateFallbacks *fallbacks,
                                         void *user)
{
  state->fallbacks = fallbacks;
  state->fbdata = user;
}

// Call initpen callback
void nvim_vterm_state_call_initpen(VTermState *state)
{
  if (state->callbacks && state->callbacks->initpen) {
    (*state->callbacks->initpen)(state->cbdata);
  }
}

// Selection info setters
void nvim_vterm_state_set_selection_callbacks_ptr(VTermState *state,
                                                   const VTermSelectionCallbacks *callbacks,
                                                   void *user,
                                                   char *buffer,
                                                   size_t buflen)
{
  state->selection.callbacks = callbacks;
  state->selection.user = user;
  state->selection.buffer = buffer;
  state->selection.buflen = buflen;
}

// Allocator helpers
void *nvim_vterm_state_malloc(VTermState *state, size_t size)
{
  return vterm_allocator_malloc(state->vt, size);
}

void nvim_vterm_state_free_ptr(VTermState *state, void *ptr)
{
  vterm_allocator_free(state->vt, ptr);
}

// VTerm tmpbuffer accessor
void *nvim_vterm_state_get_vt_tmpbuffer(const VTermState *state)
{
  return state->vt->tmpbuffer;
}

size_t nvim_vterm_state_get_vt_tmpbuffer_len(const VTermState *state)
{
  return state->vt->tmpbuffer_len;
}

int nvim_vterm_state_get_vt_mode_utf8(const VTermState *state)
{
  return state->vt->mode.utf8;
}

// Thin C wrappers for Rust pen.rs FFI exports (moved from pen.c)

// Rust FFI declarations (implemented in src/nvim-rs/vterm/src/pen.rs)
extern void rs_vterm_state_newpen(VTermState *state);
extern void rs_vterm_state_resetpen(VTermState *state);
extern void rs_vterm_state_set_default_colors(VTermState *state, const VTermColor *default_fg,
                                              const VTermColor *default_bg);
extern void rs_vterm_state_set_palette_color(VTermState *state, int index, const VTermColor *col);
extern void rs_vterm_state_convert_color_to_rgb(const VTermState *state, VTermColor *col);
extern void rs_vterm_state_savepen(VTermState *state, int save);
extern int rs_vterm_state_set_penattr(VTermState *state, int attr, int type, VTermValue *val);
extern void rs_vterm_state_setpen(VTermState *state, const long args[], int argcount);
extern int rs_vterm_state_getpen(VTermState *state, long args[], int argcount);

void vterm_state_newpen(VTermState *state) { rs_vterm_state_newpen(state); }

void vterm_state_resetpen(VTermState *state) { rs_vterm_state_resetpen(state); }

void vterm_state_savepen(VTermState *state, int save) { rs_vterm_state_savepen(state, save); }

void vterm_state_set_default_colors(VTermState *state, const VTermColor *default_fg,
                                    const VTermColor *default_bg)
{
  rs_vterm_state_set_default_colors(state, default_fg, default_bg);
}

void vterm_state_set_palette_color(VTermState *state, int index, const VTermColor *col)
{
  rs_vterm_state_set_palette_color(state, index, col);
}

void vterm_state_convert_color_to_rgb(const VTermState *state, VTermColor *col)
{
  rs_vterm_state_convert_color_to_rgb(state, col);
}

void vterm_state_setpen(VTermState *state, const long args[], int argcount)
{
  rs_vterm_state_setpen(state, args, argcount);
}

int vterm_state_getpen(VTermState *state, long args[], int argcount)
{
  return rs_vterm_state_getpen(state, args, argcount);
}

int vterm_state_set_penattr(VTermState *state, VTermAttr attr, VTermValueType type, VTermValue *val)
{
  return rs_vterm_state_set_penattr(state, (int)attr, (int)type, val);
}
