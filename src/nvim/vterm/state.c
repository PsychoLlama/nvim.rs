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

static void updatecursor(VTermState *state, VTermPos *oldpos, int cancel_phantom)
{
  if (state->pos.col == oldpos->col && state->pos.row == oldpos->row) {
    return;
  }

  if (cancel_phantom) {
    state->at_phantom = 0;
  }

  if (state->callbacks && state->callbacks->movecursor) {
    if ((*state->callbacks->movecursor)(state->pos, *oldpos, state->mode.cursor_visible,
                                        state->cbdata)) {
      return;
    }
  }
}

static void erase(VTermState *state, VTermRect rect, int selective)
{
  if (rect.end_col == state->cols) {
    // If we're erasing the final cells of any lines, cancel the continuation marker on the
    // subsequent line
    for (int row = rect.start_row + 1; row < rect.end_row + 1 && row < state->rows; row++) {
      state->lineinfo[row].continuation = 0;
    }
  }

  if (state->callbacks && state->callbacks->erase) {
    if ((*state->callbacks->erase)(rect, selective, state->cbdata)) {
      return;
    }
  }
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

static void scroll(VTermState *state, VTermRect rect, int downward, int rightward)
{
  if (!downward && !rightward) {
    return;
  }

  int rows = rect.end_row - rect.start_row;
  if (downward > rows) {
    downward = rows;
  } else if (downward < -rows) {
    downward = -rows;
  }

  int cols = rect.end_col - rect.start_col;
  if (rightward > cols) {
    rightward = cols;
  } else if (rightward < -cols) {
    rightward = -cols;
  }

  // Update lineinfo if full line
  if (rect.start_col == 0 && rect.end_col == state->cols && rightward == 0) {
    int height = rect.end_row - rect.start_row - abs(downward);

    if (downward > 0) {
      memmove(state->lineinfo + rect.start_row,
              state->lineinfo + rect.start_row + downward,
              (size_t)height * sizeof(state->lineinfo[0]));
      for (int row = rect.end_row - downward; row < rect.end_row; row++) {
        state->lineinfo[row] = (VTermLineInfo){ 0 };
      }
    } else {
      memmove(state->lineinfo + rect.start_row - downward,
              state->lineinfo + rect.start_row,
              (size_t)height * sizeof(state->lineinfo[0]));
      for (int row = rect.start_row; row < rect.start_row - downward; row++) {
        state->lineinfo[row] = (VTermLineInfo){ 0 };
      }
    }
  }

  if (state->callbacks && state->callbacks->scrollrect) {
    if ((*state->callbacks->scrollrect)(rect, downward, rightward, state->cbdata)) {
      return;
    }
  }

  if (state->callbacks) {
    vterm_scroll_rect(rect, downward, rightward,
                      state->callbacks->moverect, state->callbacks->erase, state->cbdata);
  }
}

static void linefeed(VTermState *state)
{
  if (state->pos.row == SCROLLREGION_BOTTOM(state) - 1) {
    VTermRect rect = {
      .start_row = state->scrollregion_top,
      .end_row = SCROLLREGION_BOTTOM(state),
      .start_col = SCROLLREGION_LEFT(state),
      .end_col = SCROLLREGION_RIGHT(state),
    };

    scroll(state, rect, 1, 0);
  } else if (state->pos.row < state->rows - 1) {
    state->pos.row++;
  }
}

static void set_col_tabstop(VTermState *state, int col)
{
  uint8_t mask = (uint8_t)(1 << (col & 7));
  state->tabstops[col >> 3] |= mask;
}

static void clear_col_tabstop(VTermState *state, int col)
{
  uint8_t mask = (uint8_t)(1 << (col & 7));
  state->tabstops[col >> 3] &= ~mask;
}

static int is_col_tabstop(VTermState *state, int col)
{
  uint8_t mask = (uint8_t)(1 << (col & 7));
  return state->tabstops[col >> 3] & mask;
}

static int is_cursor_in_scrollregion(const VTermState *state)
{
  if (state->pos.row < state->scrollregion_top
      || state->pos.row >= SCROLLREGION_BOTTOM(state)) {
    return 0;
  }
  if (state->pos.col < SCROLLREGION_LEFT(state)
      || state->pos.col >= SCROLLREGION_RIGHT(state)) {
    return 0;
  }

  return 1;
}

static void tab(VTermState *state, int count, int direction)
{
  while (count > 0) {
    if (direction > 0) {
      if (state->pos.col >= THISROWWIDTH(state) - 1) {
        return;
      }

      state->pos.col++;
    } else if (direction < 0) {
      if (state->pos.col < 1) {
        return;
      }

      state->pos.col--;
    }

    if (is_col_tabstop(state, state->pos.col)) {
      count--;
    }
  }
}

#define NO_FORCE 0
#define FORCE    1

#define DWL_OFF 0
#define DWL_ON  1

#define DHL_OFF    0
#define DHL_TOP    1
#define DHL_BOTTOM 2

static void set_lineinfo(VTermState *state, int row, int force, int dwl, int dhl)
{
  rs_vterm_state_set_lineinfo(state, row, force, dwl, dhl);
}

static int on_text(const char bytes[], size_t len, void *user)
{
  VTermState *state = user;

  VTermPos oldpos = state->pos;

  uint32_t *codepoints = (uint32_t *)(state->vt->tmpbuffer);
  size_t maxpoints = (state->vt->tmpbuffer_len) / sizeof(uint32_t);

  int npoints = 0;
  size_t eaten = 0;

  VTermEncodingInstance *encoding =
    state->gsingle_set ? &state->encoding[state->gsingle_set]
                       : !(bytes[eaten] & 0x80) ? &state->encoding[state->gl_set]
                                                : state->vt->mode.utf8 ? &state->encoding_utf8
                                                                       : &state->encoding[state->
                                                                                          gr_set];

  (*encoding->enc->decode)(encoding->enc, encoding->data,
                           codepoints, &npoints, state->gsingle_set ? 1 : (int)maxpoints,
                           bytes, &eaten, len);

  // There's a chance an encoding (e.g. UTF-8) hasn't found enough bytes yet for even a single codepoint
  if (!npoints) {
    return (int)eaten;
  }

  if (state->gsingle_set && npoints) {
    state->gsingle_set = 0;
  }

  int i = 0;
  GraphemeState grapheme_state = GRAPHEME_STATE_INIT;
  size_t grapheme_len = 0;
  bool recombine = false;

  // See if the cursor has moved since
  if (state->pos.row == state->combine_pos.row
      && state->pos.col == state->combine_pos.col + state->combine_width) {
    // This is a combining char. that needs to be merged with the previous glyph output
    if (utf_iscomposing((int)state->grapheme_last, (int)codepoints[i], &state->grapheme_state)) {
      // Find where we need to append these combining chars
      grapheme_len = state->grapheme_len;
      grapheme_state = state->grapheme_state;
      state->pos.col = state->combine_pos.col;
      recombine = true;
    } else {
      DEBUG_LOG("libvterm: TODO: Skip over split char+combining\n");
    }
  }

  while (i < npoints) {
    // Try to find combining characters following this
    do {
      if (grapheme_len < sizeof(state->grapheme_buf) - 4) {
        grapheme_len += (size_t)utf_char2bytes((int)codepoints[i],
                                               state->grapheme_buf + grapheme_len);
      }
      i++;
    } while (i < npoints && utf_iscomposing((int)codepoints[i - 1], (int)codepoints[i],
                                            &grapheme_state));

    int width = utf_ptr2cells_len(state->grapheme_buf, (int)grapheme_len);

    if (state->at_phantom || state->pos.col + width > THISROWWIDTH(state)) {
      linefeed(state);
      state->pos.col = 0;
      state->at_phantom = 0;
      state->lineinfo[state->pos.row].continuation = 1;
    }

    if (state->mode.insert && !recombine) {
      // TODO(vterm): This will be a little inefficient for large bodies of text, as it'll have to
      // 'ICH' effectively before every glyph. We should scan ahead and ICH as many times as
      // required
      VTermRect rect = {
        .start_row = state->pos.row,
        .end_row = state->pos.row + 1,
        .start_col = state->pos.col,
        .end_col = THISROWWIDTH(state),
      };
      scroll(state, rect, 0, -1);
    }

    schar_T sc = schar_from_buf(state->grapheme_buf, grapheme_len);
    putglyph(state, sc, width, state->pos);

    if (i == npoints) {
      // End of the buffer. Save the chars in case we have to combine with more on the next call
      state->grapheme_len = grapheme_len;
      state->grapheme_last = codepoints[i - 1];
      state->grapheme_state = grapheme_state;
      state->combine_width = width;
      state->combine_pos = state->pos;
    } else {
      grapheme_len = 0;
      recombine = false;
    }

    if (state->pos.col + width >= THISROWWIDTH(state)) {
      if (state->mode.autowrap) {
        state->at_phantom = 1;
      }
    } else {
      state->pos.col += width;
    }
  }

  updatecursor(state, &oldpos, 0);

#ifdef DEBUG
  if (state->pos.row < 0 || state->pos.row >= state->rows
      || state->pos.col < 0 || state->pos.col >= state->cols) {
    fprintf(stderr, "Position out of bounds after text: (%d,%d)\n",
            state->pos.row, state->pos.col);
    abort();
  }
#endif

  return (int)eaten;
}

static int on_control(uint8_t control, void *user)
{
  VTermState *state = user;
  return rs_vterm_state_on_control(state, control);
}

static int settermprop_bool(VTermState *state, VTermProp prop, int v)
{
  VTermValue val = { .boolean = v };
  return vterm_state_set_termprop(state, prop, &val);
}

static int settermprop_int(VTermState *state, VTermProp prop, int v)
{
  VTermValue val = { .number = v };
  return vterm_state_set_termprop(state, prop, &val);
}

static int settermprop_string(VTermState *state, VTermProp prop, VTermStringFragment frag)
{
  VTermValue val = { .string = frag };
  return vterm_state_set_termprop(state, prop, &val);
}

static void savecursor(VTermState *state, int save)
{
  rs_vterm_state_savecursor(state, save);
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
  VTermPos oldpos = state->pos;

  if (cols != state->cols) {
    uint8_t *newtabstops = vterm_allocator_malloc(state->vt, ((size_t)cols + 7) / 8);

    // TODO(vterm): This can all be done much more efficiently bytewise
    int col;
    for (col = 0; col < state->cols && col < cols; col++) {
      uint8_t mask = (uint8_t)(1 << (col & 7));
      if (state->tabstops[col >> 3] & mask) {
        newtabstops[col >> 3] |= mask;
      } else {
        newtabstops[col >> 3] &= ~mask;
      }
    }

    for (; col < cols; col++) {
      uint8_t mask = (uint8_t)(1 << (col & 7));
      if (col % 8 == 0) {
        newtabstops[col >> 3] |= mask;
      } else {
        newtabstops[col >> 3] &= ~mask;
      }
    }

    vterm_allocator_free(state->vt, state->tabstops);
    state->tabstops = newtabstops;
  }

  state->rows = rows;
  state->cols = cols;

  if (state->scrollregion_bottom > -1) {
    UBOUND(state->scrollregion_bottom, state->rows);
  }
  if (state->scrollregion_right > -1) {
    UBOUND(state->scrollregion_right, state->cols);
  }

  VTermStateFields fields = {
    .pos = state->pos,
    .lineinfos = {[0] = state->lineinfos[0], [1] = state->lineinfos[1] },
  };

  if (state->callbacks && state->callbacks->resize) {
    (*state->callbacks->resize)(rows, cols, &fields, state->cbdata);
    state->pos = fields.pos;

    state->lineinfos[0] = fields.lineinfos[0];
    state->lineinfos[1] = fields.lineinfos[1];
  } else {
    if (rows != state->rows) {
      for (int bufidx = BUFIDX_PRIMARY; bufidx <= BUFIDX_ALTSCREEN; bufidx++) {
        VTermLineInfo *oldlineinfo = state->lineinfos[bufidx];
        if (!oldlineinfo) {
          continue;
        }

        VTermLineInfo *newlineinfo = vterm_allocator_malloc(state->vt,
                                                            (size_t)rows * sizeof(VTermLineInfo));

        int row;
        for (row = 0; row < state->rows && row < rows; row++) {
          newlineinfo[row] = oldlineinfo[row];
        }

        for (; row < rows; row++) {
          newlineinfo[row] = (VTermLineInfo){
            .doublewidth = 0,
          };
        }

        vterm_allocator_free(state->vt, state->lineinfos[bufidx]);
        state->lineinfos[bufidx] = newlineinfo;
      }
    }
  }

  state->lineinfo = state->lineinfos[state->mode.alt_screen ? BUFIDX_ALTSCREEN : BUFIDX_PRIMARY];

  if (state->at_phantom && state->pos.col < cols - 1) {
    state->at_phantom = 0;
    state->pos.col++;
  }

  if (state->pos.row < 0) {
    state->pos.row = 0;
  }
  if (state->pos.row >= rows) {
    state->pos.row = rows - 1;
  }
  if (state->pos.col < 0) {
    state->pos.col = 0;
  }
  if (state->pos.col >= cols) {
    state->pos.col = cols - 1;
  }

  updatecursor(state, &oldpos, 1);

  return 1;
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
  state->scrollregion_top = 0;
  state->scrollregion_bottom = -1;
  state->scrollregion_left = 0;
  state->scrollregion_right = -1;

  state->mode.keypad = 0;
  state->mode.cursor = 0;
  state->mode.autowrap = 1;
  state->mode.insert = 0;
  state->mode.newline = 0;
  state->mode.alt_screen = 0;
  state->mode.origin = 0;
  state->mode.leftrightmargin = 0;
  state->mode.bracketpaste = 0;
  state->mode.report_focus = 0;

  state->mouse_flags = 0;

  state->vt->mode.ctrl8bit = 0;

  for (int col = 0; col < state->cols; col++) {
    if (col % 8 == 0) {
      set_col_tabstop(state, col);
    } else {
      clear_col_tabstop(state, col);
    }
  }

  for (int row = 0; row < state->rows; row++) {
    set_lineinfo(state, row, FORCE, DWL_OFF, DHL_OFF);
  }

  if (state->callbacks && state->callbacks->initpen) {
    (*state->callbacks->initpen)(state->cbdata);
  }

  vterm_state_resetpen(state);

  VTermEncoding *default_enc = state->vt->mode.utf8
                               ? rs_vterm_lookup_encoding_ptr(ENC_UTF8,      'u')
                               : rs_vterm_lookup_encoding_ptr(ENC_SINGLE_94, 'B');

  for (int i = 0; i < 4; i++) {
    state->encoding[i].enc = default_enc;
    if (default_enc->init) {
      (*default_enc->init)(default_enc, state->encoding[i].data);
    }
  }

  state->gl_set = 0;
  state->gr_set = 1;
  state->gsingle_set = 0;

  state->protected_cell = 0;

  // Initialise the props
  settermprop_bool(state, VTERM_PROP_CURSORVISIBLE, 1);
  settermprop_bool(state, VTERM_PROP_CURSORBLINK,   1);
  settermprop_int(state, VTERM_PROP_CURSORSHAPE,   VTERM_PROP_CURSORSHAPE_BLOCK);

  if (hard) {
    state->pos.row = 0;
    state->pos.col = 0;
    state->at_phantom = 0;

    VTermRect rect = { 0, state->rows, 0, state->cols };
    erase(state, rect, 0);
  }
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
  if (fallbacks) {
    state->fallbacks = fallbacks;
    state->fbdata = user;
  } else {
    state->fallbacks = NULL;
    state->fbdata = NULL;
  }
}

int vterm_state_set_termprop(VTermState *state, VTermProp prop, VTermValue *val)
{
  // Only store the new value of the property if usercode said it was happy. This is especially
  // important for altscreen switching
  if (state->callbacks && state->callbacks->settermprop) {
    if (!(*state->callbacks->settermprop)(prop, val, state->cbdata)) {
      return 0;
    }
  }

  switch (prop) {
  case VTERM_PROP_TITLE:
  case VTERM_PROP_ICONNAME:
    // we don't store these, just transparently pass through
    return 1;
  case VTERM_PROP_CURSORVISIBLE:
    state->mode.cursor_visible = (unsigned)val->boolean;
    return 1;
  case VTERM_PROP_CURSORBLINK:
    state->mode.cursor_blink = (unsigned)val->boolean;
    return 1;
  case VTERM_PROP_CURSORSHAPE:
    state->mode.cursor_shape = (unsigned)val->number;
    return 1;
  case VTERM_PROP_REVERSE:
    state->mode.screen = (unsigned)val->boolean;
    return 1;
  case VTERM_PROP_ALTSCREEN:
    state->mode.alt_screen = (unsigned)val->boolean;
    state->lineinfo = state->lineinfos[state->mode.alt_screen ? BUFIDX_ALTSCREEN : BUFIDX_PRIMARY];
    if (state->mode.alt_screen) {
      VTermRect rect = {
        .start_row = 0,
        .start_col = 0,
        .end_row = state->rows,
        .end_col = state->cols,
      };
      erase(state, rect, 0);
    }
    return 1;
  case VTERM_PROP_MOUSE:
    state->mouse_flags = 0;
    if (val->number) {
      state->mouse_flags |= MOUSE_WANT_CLICK;
    }
    if (val->number == VTERM_PROP_MOUSE_DRAG) {
      state->mouse_flags |= MOUSE_WANT_DRAG;
    }
    if (val->number == VTERM_PROP_MOUSE_MOVE) {
      state->mouse_flags |= MOUSE_WANT_MOVE;
    }
    return 1;
  case VTERM_PROP_FOCUSREPORT:
    state->mode.report_focus = (unsigned)val->boolean;
    return 1;
  case VTERM_PROP_THEMEUPDATES:
    state->mode.theme_updates = (unsigned)val->boolean;
    return 1;

  case VTERM_N_PROPS:
    return 0;
  }

  return 0;
}

void vterm_state_focus_in(VTermState *state)
{
  if (state->mode.report_focus) {
    vterm_push_output_sprintf_ctrl(state->vt, C1_CSI, "I");
  }
}

void vterm_state_focus_out(VTermState *state)
{
  if (state->mode.report_focus) {
    vterm_push_output_sprintf_ctrl(state->vt, C1_CSI, "O");
  }
}

const VTermLineInfo *vterm_state_get_lineinfo(const VTermState *state, int row) { return state->lineinfo + row; }

void vterm_state_set_selection_callbacks(VTermState *state,
                                         const VTermSelectionCallbacks *callbacks, void *user,
                                         char *buffer, size_t buflen)
{
  if (buflen && !buffer) {
    buffer = vterm_allocator_malloc(state->vt, buflen);
  }

  state->selection.callbacks = callbacks;
  state->selection.user = user;
  state->selection.buffer = buffer;
  state->selection.buflen = buflen;
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
