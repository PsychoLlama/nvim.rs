// register.c: functions for managing registers

#include <stdbool.h>

#include "nvim/api/private/helpers.h"
#include "nvim/autocmd.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/buffer_updates.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/clipboard.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/ex_cmds2.h"
#include "nvim/ex_getln.h"
#include "nvim/extmark.h"
#include "nvim/file_search.h"
#include "nvim/fold.h"
#include "nvim/garray.h"
#include "nvim/getchar.h"
#include "nvim/globals.h"
#include "nvim/indent.h"
#include "nvim/memory.h"
#include "nvim/keycodes.h"
#include "nvim/mark.h"
#include "nvim/mbyte.h"
#include "nvim/memline.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_vars.h"
#include "nvim/os/input.h"
#include "nvim/os/time.h"
#include "nvim/plines.h"
#include "nvim/register.h"
#include "nvim/search.h"
#include "nvim/strings.h"
#include "nvim/terminal.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/undo.h"

#include "register.c.generated.h"

// Global register state owned by Rust (src/nvim-rs/register/src/lib.rs).
extern yankreg_T y_regs[NUM_REGISTERS];
extern yankreg_T *y_previous;
extern char *expr_line;
extern int execreg_lastc;

/// Set curwin->w_alt_fnum from a buf_T pointer (called from Rust write_reg_contents_ex).
void nvim_register_set_alt_fnum(buf_T *buf) { curwin->w_alt_fnum = buf->b_fnum; }

// Functions now implemented in Rust (src/nvim-rs/register/src/lib.rs) with #[export_name].
extern int stuff_yank(int regname, char *p);

// C accessor wrappers used by Rust crates via FFI.
// These thin wrappers delegate to Neovim's memory allocator and error helpers.

/// Free a register's contents (delegating to the Rust free_register export).
extern void free_register(yankreg_T *reg);
void nvim_free_register(yankreg_T *reg) { free_register(reg); }

/// Generic xfree wrapper used by Rust crates that cannot call xfree directly.
void nvim_xfree(void *ptr) { xfree(ptr); }

/// Generic xstrdup wrapper.
char *nvim_xstrdup(const char *str) { return xstrdup(str); }

/// Generic xmalloc wrapper.
void *nvim_xmalloc(size_t size) { return xmalloc(size); }

/// Generic xcalloc wrapper.
void *nvim_xcalloc(size_t count, size_t size) { return xcalloc(count, size); }

/// Generic xrealloc wrapper.
void *nvim_xrealloc(void *ptr, size_t size) { return xrealloc(ptr, size); }

/// Generic xmallocz wrapper (allocates size+1 bytes zeroed at end).
char *nvim_xmallocz(size_t size) { return xmallocz(size); }

/// Emit "E35: No previous regular expression" error.
void nvim_emsg_noprevre(void) { emsg(_(e_noprevre)); }

/// Emit "E29: No inserted text yet" error.
void nvim_emsg_noinstext(void) { emsg(_(e_noinstext)); }

/// Emit "E30: No previous command line" error.
void nvim_emsg_nolastcmd(void) { emsg(_(e_nolastcmd)); }

/// Return curbuf->b_fname (for Rust get_spec_reg).
char *nvim_register_get_curbuf_fname(void) { return curbuf->b_fname; }

/// Return curwin (for Rust op_yank_reg update_topline call).
void *nvim_register_get_curwin(void) { return curwin; }

/// cbuf_as_string is a macro; provide a real function for Rust FFI.
String nvim_register_cbuf_as_string(char *buf, size_t len)
{
  return cbuf_as_string(buf, len);
}

/// Copy curwin->w_cursor into *pos (for Rust insert_reg).
void nvim_register_get_curwin_cursor(pos_T *pos) { *pos = curwin->w_cursor; }

/// Set curwin->w_cursor from *pos (for Rust insert_reg).
void nvim_register_set_curwin_cursor(const pos_T *pos) { curwin->w_cursor = *pos; }

/// Return the global State variable (for Rust insert_reg).
int nvim_register_get_State(void) { return State; }

/// Return ml_get_buf(curwin->w_buffer, curwin->w_cursor.lnum) (for Rust get_spec_reg).
char *nvim_register_ml_get_buf_curwin_lnum(void)
{
  return ml_get_buf(curwin->w_buffer, curwin->w_cursor.lnum);
}

// do_record is implemented in Rust (src/nvim-rs/register/src/lib.rs).

// put_in_typebuf, put_reedit_in_typebuf, execreg_line_continuation are
// private helpers implemented in Rust (src/nvim-rs/register/src/lib.rs).
// do_execreg is implemented in Rust with export_name = "do_execreg".

// insert_reg and get_spec_reg are implemented in Rust
// (src/nvim-rs/register/src/lib.rs) with export_name.

// op_yank, op_yank_reg, yank_copy_line, do_autocmd_textyankpost are implemented
// in Rust (src/nvim-rs/register/src/lib.rs) with export_name.

// --- oparg_T accessors for Rust op_yank_reg ---
int nvim_oap_get_motion_type(oparg_T *oap) { return (int)oap->motion_type; }
void nvim_oap_get_start(oparg_T *oap, pos_T *pos) { *pos = oap->start; }
void nvim_oap_get_end(oparg_T *oap, pos_T *pos) { *pos = oap->end; }
bool nvim_oap_get_inclusive(oparg_T *oap) { return oap->inclusive; }
bool nvim_oap_get_is_VIsual(oparg_T *oap) { return oap->is_VIsual; }
int nvim_oap_get_line_count(oparg_T *oap) { return (int)oap->line_count; }
int nvim_oap_get_start_vcol(oparg_T *oap) { return (int)oap->start_vcol; }
int nvim_oap_get_end_vcol(oparg_T *oap) { return (int)oap->end_vcol; }
int nvim_oap_get_regname(oparg_T *oap) { return oap->regname; }
bool nvim_oap_get_excl_tr_ws(oparg_T *oap) { return oap->excl_tr_ws; }
int nvim_oap_get_op_type(oparg_T *oap) { return oap->op_type; }

// --- Yank message (NGETTEXT cannot be used from Rust) ---
void nvim_register_yank_msg(size_t yanklines, const char *namebuf, bool is_block)
{
  if (is_block) {
    smsg(0, NGETTEXT("block of %" PRId64 " line yanked%s",
                     "block of %" PRId64 " lines yanked%s", yanklines),
         (int64_t)yanklines, namebuf);
  } else {
    smsg(0, NGETTEXT("%" PRId64 " line yanked%s",
                     "%" PRId64 " lines yanked%s", yanklines),
         (int64_t)yanklines, namebuf);
  }
}

// --- Yank name buffer: fills buf with " into "regname or empty string ---
void nvim_register_yank_namebuf(int regname, char *buf, size_t bufsz)
{
  if (regname == NUL) {
    buf[0] = NUL;
  } else {
    vim_snprintf(buf, bufsz, _(" into \"%c"), regname);
  }
}

// --- Option accessors ---
int nvim_register_get_p_sel_char(void) { return *p_sel; }
int64_t nvim_register_get_p_report(void) { return p_report; }
bool nvim_register_p_cpo_has_regappend(void)
{
  return vim_strchr(p_cpo, CPO_REGAPPEND) != NULL;
}
bool nvim_register_cmod_lockmarks(void)
{
  return (cmdmod.cmod_flags & CMOD_LOCKMARKS) != 0;
}

// --- curwin/curbuf accessors ---
int nvim_register_get_curwin_curswant(void) { return (int)curwin->w_curswant; }
void nvim_register_curbuf_set_op_start(const pos_T *pos) { curbuf->b_op_start = *pos; }
void nvim_register_curbuf_set_op_end(const pos_T *pos) { curbuf->b_op_end = *pos; }
void nvim_register_curbuf_set_op_start_col(int col) { curbuf->b_op_start.col = (colnr_T)col; }
void nvim_register_curbuf_set_op_end_col(int col) { curbuf->b_op_end.col = (colnr_T)col; }
void nvim_register_curbuf_decl_op_end(void) { decl(&curbuf->b_op_end); }

// --- tv_list_set_lock wrapper (inline in C, needs wrapper for Rust) ---
void nvim_register_tv_list_set_lock_fixed(list_T *list) { tv_list_set_lock(list, VAR_FIXED); }

// --- textlock inc/dec for do_autocmd_textyankpost ---
// (already exist as nvim_inc_textlock / nvim_dec_textlock in ex_getln.c)

/// Put contents of register "regname" into the text.
/// Caller must check "regname" to be valid!
///
/// @param flags  PUT_FIXINDENT     make indent look nice
///               PUT_CURSEND       leave cursor after end of new text
///               PUT_LINE          force linewise put (":put")
///               PUT_BLOCK_INNER   in block mode, do not add trailing spaces
/// @param dir    BACKWARD for 'P', FORWARD for 'p'
void do_put(int regname, yankreg_T *reg, int dir, int count, int flags)
{
  size_t totlen = 0;  // init for gcc
  linenr_T lnum = 0;
  MotionType y_type;
  size_t y_size;
  int y_width = 0;
  colnr_T vcol = 0;
  String *y_array = NULL;
  linenr_T nr_lines = 0;
  bool allocated = false;
  const pos_T orig_start = curbuf->b_op_start;
  const pos_T orig_end = curbuf->b_op_end;
  unsigned cur_ve_flags = get_ve_flags(curwin);

  curbuf->b_op_start = curwin->w_cursor;        // default for '[ mark
  curbuf->b_op_end = curwin->w_cursor;          // default for '] mark

  // Using inserted text works differently, because the register includes
  // special characters (newlines, etc.).
  if (regname == '.' && !reg) {
    bool non_linewise_vis = (VIsual_active && VIsual_mode != 'V');

    // PUT_LINE has special handling below which means we use 'i' to start.
    char command_start_char = non_linewise_vis
                              ? 'c'
                              : (flags & PUT_LINE ? 'i' : (dir == FORWARD ? 'a' : 'i'));

    // To avoid 'autoindent' on linewise puts, create a new line with `:put _`.
    if (flags & PUT_LINE) {
      do_put('_', NULL, dir, 1, PUT_LINE);
    }

    // If given a count when putting linewise, we stuff the readbuf with the
    // dot register 'count' times split by newlines.
    if (flags & PUT_LINE) {
      stuffcharReadbuff(command_start_char);
      for (; count > 0; count--) {
        stuff_inserted(NUL, 1, count != 1);
        if (count != 1) {
          // To avoid 'autoindent' affecting the text, use Ctrl_U to remove any
          // whitespace. Can't just insert Ctrl_U into readbuf1, this would go
          // back to the previous line in the case of 'noautoindent' and
          // 'backspace' includes "eol". So we insert a dummy space for Ctrl_U
          // to consume.
          stuffReadbuff("\n ");
          stuffcharReadbuff(Ctrl_U);
        }
      }
    } else {
      stuff_inserted(command_start_char, count, false);
    }

    // Putting the text is done later, so can't move the cursor to the next
    // character.  Simulate it with motion commands after the insert.
    if (flags & PUT_CURSEND) {
      if (flags & PUT_LINE) {
        stuffReadbuff("j0");
      } else {
        // Avoid ringing the bell from attempting to move into the space after
        // the current line. We can stuff the readbuffer with "l" if:
        // 1) 'virtualedit' is "all" or "onemore"
        // 2) We are not at the end of the line
        // 3) We are not  (one past the end of the line && on the last line)
        //    This allows a visual put over a selection one past the end of the
        //    line joining the current line with the one below.

        // curwin->w_cursor.col marks the byte position of the cursor in the
        // currunt line. It increases up to a max of
        // strlen(ml_get(curwin->w_cursor.lnum)). With 'virtualedit' and the
        // cursor past the end of the line, curwin->w_cursor.coladd is
        // incremented instead of curwin->w_cursor.col.
        char *cursor_pos = get_cursor_pos_ptr();
        bool one_past_line = (*cursor_pos == NUL);
        bool eol = false;
        if (!one_past_line) {
          eol = (*(cursor_pos + utfc_ptr2len(cursor_pos)) == NUL);
        }

        bool ve_allows = (cur_ve_flags == kOptVeFlagAll || cur_ve_flags == kOptVeFlagOnemore);
        bool eof = curbuf->b_ml.ml_line_count == curwin->w_cursor.lnum
                   && one_past_line;
        if (ve_allows || !(eol || eof)) {
          stuffcharReadbuff('l');
        }
      }
    } else if (flags & PUT_LINE) {
      stuffReadbuff("g'[");
    }

    // So the 'u' command restores cursor position after ".p, save the cursor
    // position now (though not saving any text).
    if (command_start_char == 'a') {
      if (u_save(curwin->w_cursor.lnum, curwin->w_cursor.lnum + 1) == FAIL) {
        return;
      }
    }
    return;
  }

  // For special registers '%' (file name), '#' (alternate file name) and
  // ':' (last command line), etc. we have to create a fake yank register.
  String insert_string = STRING_INIT;
  if (!reg && get_spec_reg(regname, &insert_string.data, &allocated, true)) {
    if (insert_string.data == NULL) {
      return;
    }
  }

  if (!curbuf->terminal) {
    // Autocommands may be executed when saving lines for undo.  This might
    // make y_array invalid, so we start undo now to avoid that.
    if (u_save(curwin->w_cursor.lnum, curwin->w_cursor.lnum + 1) == FAIL) {
      return;
    }
  }

  if (insert_string.data != NULL) {
    insert_string.size = strlen(insert_string.data);
    y_type = kMTCharWise;
    if (regname == '=') {
      // For the = register we need to split the string at NL
      // characters.
      // Loop twice: count the number of lines and save them.
      while (true) {
        y_size = 0;
        char *ptr = insert_string.data;
        size_t ptrlen = insert_string.size;
        while (ptr != NULL) {
          if (y_array != NULL) {
            y_array[y_size].data = ptr;
          }
          y_size++;
          char *tmp = vim_strchr(ptr, '\n');
          if (tmp == NULL) {
            if (y_array != NULL) {
              y_array[y_size - 1].size = ptrlen;
            }
          } else {
            if (y_array != NULL) {
              *tmp = NUL;
              y_array[y_size - 1].size = (size_t)(tmp - ptr);
              ptrlen -= y_array[y_size - 1].size + 1;
            }
            tmp++;
            // A trailing '\n' makes the register linewise.
            if (*tmp == NUL) {
              y_type = kMTLineWise;
              break;
            }
          }
          ptr = tmp;
        }
        if (y_array != NULL) {
          break;
        }
        y_array = xmalloc(y_size * sizeof(String));
      }
    } else {
      y_size = 1;               // use fake one-line yank register
      y_array = &insert_string;
    }
  } else {
    // in case of replacing visually selected text
    // the yankreg might already have been saved to avoid
    // just restoring the deleted text.
    if (reg == NULL) {
      reg = get_yank_register(regname, YREG_PASTE);
    }

    y_type = reg->y_type;
    y_width = reg->y_width;
    y_size = reg->y_size;
    y_array = reg->y_array;
  }

  if (curbuf->terminal) {
    terminal_paste(count, y_array, y_size);
    return;
  }

  colnr_T split_pos = 0;
  if (y_type == kMTLineWise) {
    if (flags & PUT_LINE_SPLIT) {
      // "p" or "P" in Visual mode: split the lines to put the text in
      // between.
      if (u_save_cursor() == FAIL) {
        goto end;
      }
      char *curline = get_cursor_line_ptr();
      char *p = get_cursor_pos_ptr();
      char *const p_orig = p;
      const size_t plen = (size_t)get_cursor_pos_len();
      if (dir == FORWARD && *p != NUL) {
        MB_PTR_ADV(p);
      }
      // we need this later for the correct extmark_splice() event
      split_pos = (colnr_T)(p - curline);

      char *ptr = xmemdupz(p, plen - (size_t)(p - p_orig));
      ml_append(curwin->w_cursor.lnum, ptr, 0, false);
      xfree(ptr);

      ptr = xmemdupz(get_cursor_line_ptr(), (size_t)split_pos);
      ml_replace(curwin->w_cursor.lnum, ptr, false);
      nr_lines++;
      dir = FORWARD;

      buf_updates_send_changes(curbuf, curwin->w_cursor.lnum, 1, 1);
    }
    if (flags & PUT_LINE_FORWARD) {
      // Must be "p" for a Visual block, put lines below the block.
      curwin->w_cursor = curbuf->b_visual.vi_end;
      dir = FORWARD;
    }
    curbuf->b_op_start = curwin->w_cursor;      // default for '[ mark
    curbuf->b_op_end = curwin->w_cursor;        // default for '] mark
  }

  if (flags & PUT_LINE) {  // :put command or "p" in Visual line mode.
    y_type = kMTLineWise;
  }

  if (y_size == 0 || y_array == NULL) {
    semsg(_("E353: Nothing in register %s"),
          regname == 0 ? "\"" : transchar(regname));
    goto end;
  }

  if (y_type == kMTBlockWise) {
    lnum = curwin->w_cursor.lnum + (linenr_T)y_size + 1;
    lnum = MIN(lnum, curbuf->b_ml.ml_line_count + 1);
    if (u_save(curwin->w_cursor.lnum - 1, lnum) == FAIL) {
      goto end;
    }
  } else if (y_type == kMTLineWise) {
    lnum = curwin->w_cursor.lnum;
    // Correct line number for closed fold.  Don't move the cursor yet,
    // u_save() uses it.
    if (dir == BACKWARD) {
      hasFolding(curwin, lnum, &lnum, NULL);
    } else {
      hasFolding(curwin, lnum, NULL, &lnum);
    }
    if (dir == FORWARD) {
      lnum++;
    }
    // In an empty buffer the empty line is going to be replaced, include
    // it in the saved lines.
    if ((buf_is_empty(curbuf)
         ? u_save(0, 2) : u_save(lnum - 1, lnum)) == FAIL) {
      goto end;
    }
    if (dir == FORWARD) {
      curwin->w_cursor.lnum = lnum - 1;
    } else {
      curwin->w_cursor.lnum = lnum;
    }
    curbuf->b_op_start = curwin->w_cursor;      // for mark_adjust()
  } else if (u_save_cursor() == FAIL) {
    goto end;
  }

  if (cur_ve_flags == kOptVeFlagAll && y_type == kMTCharWise) {
    if (gchar_cursor() == TAB) {
      int viscol = getviscol();
      OptInt ts = curbuf->b_p_ts;
      // Don't need to insert spaces when "p" on the last position of a
      // tab or "P" on the first position.
      if (dir == FORWARD
          ? tabstop_padding(viscol, ts, curbuf->b_p_vts_array) != 1
          : curwin->w_cursor.coladd > 0) {
        coladvance_force(viscol);
      } else {
        curwin->w_cursor.coladd = 0;
      }
    } else if (curwin->w_cursor.coladd > 0 || gchar_cursor() == NUL) {
      coladvance_force(getviscol() + (dir == FORWARD));
    }
  }

  lnum = curwin->w_cursor.lnum;
  colnr_T col = curwin->w_cursor.col;

  // Block mode
  if (y_type == kMTBlockWise) {
    int incr = 0;
    struct block_def bd;
    int c = gchar_cursor();
    colnr_T endcol2 = 0;

    if (dir == FORWARD && c != NUL) {
      if (cur_ve_flags == kOptVeFlagAll) {
        getvcol(curwin, &curwin->w_cursor, &col, NULL, &endcol2);
      } else {
        getvcol(curwin, &curwin->w_cursor, NULL, NULL, &col);
      }

      // move to start of next multi-byte character
      curwin->w_cursor.col += utfc_ptr2len(get_cursor_pos_ptr());
      col++;
    } else {
      getvcol(curwin, &curwin->w_cursor, &col, NULL, &endcol2);
    }

    col += curwin->w_cursor.coladd;
    if (cur_ve_flags == kOptVeFlagAll
        && (curwin->w_cursor.coladd > 0 || endcol2 == curwin->w_cursor.col)) {
      if (dir == FORWARD && c == NUL) {
        col++;
      }
      if (dir != FORWARD && c != NUL && curwin->w_cursor.coladd > 0) {
        curwin->w_cursor.col++;
      }
      if (c == TAB) {
        if (dir == BACKWARD && curwin->w_cursor.col) {
          curwin->w_cursor.col--;
        }
        if (dir == FORWARD && col - 1 == endcol2) {
          curwin->w_cursor.col++;
        }
      }
    }
    curwin->w_cursor.coladd = 0;
    bd.textcol = 0;
    for (size_t i = 0; i < y_size; i++) {
      int spaces = 0;
      char shortline;
      // can just be 0 or 1, needed for blockwise paste beyond the current
      // buffer end
      int lines_appended = 0;

      bd.startspaces = 0;
      bd.endspaces = 0;
      vcol = 0;
      int delcount = 0;

      // add a new line
      if (curwin->w_cursor.lnum > curbuf->b_ml.ml_line_count) {
        if (ml_append(curbuf->b_ml.ml_line_count, "", 1, false) == FAIL) {
          break;
        }
        nr_lines++;
        lines_appended = 1;
      }
      // get the old line and advance to the position to insert at
      char *oldp = get_cursor_line_ptr();
      colnr_T oldlen = get_cursor_line_len();

      CharsizeArg csarg;
      CSType cstype = init_charsize_arg(&csarg, curwin, curwin->w_cursor.lnum, oldp);
      StrCharInfo ci = utf_ptr2StrCharInfo(oldp);
      vcol = 0;
      while (vcol < col && *ci.ptr != NUL) {
        incr = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &csarg).width;
        vcol += incr;
        ci = utfc_next(ci);
      }
      char *ptr = ci.ptr;
      bd.textcol = (colnr_T)(ptr - oldp);

      shortline = (vcol < col) || (vcol == col && !*ptr);

      if (vcol < col) {     // line too short, pad with spaces
        bd.startspaces = col - vcol;
      } else if (vcol > col) {
        bd.endspaces = vcol - col;
        bd.startspaces = incr - bd.endspaces;
        bd.textcol--;
        delcount = 1;
        bd.textcol -= utf_head_off(oldp, oldp + bd.textcol);
        if (oldp[bd.textcol] != TAB) {
          // Only a Tab can be split into spaces.  Other
          // characters will have to be moved to after the
          // block, causing misalignment.
          delcount = 0;
          bd.endspaces = 0;
        }
      }

      const int yanklen = (int)y_array[i].size;

      if ((flags & PUT_BLOCK_INNER) == 0) {
        // calculate number of spaces required to fill right side of block
        spaces = y_width + 1;

        cstype = init_charsize_arg(&csarg, curwin, 0, y_array[i].data);
        ci = utf_ptr2StrCharInfo(y_array[i].data);
        while (*ci.ptr != NUL) {
          spaces -= win_charsize(cstype, 0, ci.ptr, ci.chr.value, &csarg).width;
          ci = utfc_next(ci);
        }
        spaces = MAX(spaces, 0);
      }

      // Insert the new text.
      // First check for multiplication overflow.
      if (yanklen + spaces != 0
          && count > ((INT_MAX - (bd.startspaces + bd.endspaces)) / (yanklen + spaces))) {
        emsg(_(e_resulting_text_too_long));
        break;
      }

      totlen = (size_t)count * (size_t)(yanklen + spaces) + (size_t)bd.startspaces +
               (size_t)bd.endspaces;
      char *newp = xmalloc(totlen + (size_t)oldlen + 1);

      // copy part up to cursor to new line
      ptr = newp;
      memmove(ptr, oldp, (size_t)bd.textcol);
      ptr += bd.textcol;

      // may insert some spaces before the new text
      memset(ptr, ' ', (size_t)bd.startspaces);
      ptr += bd.startspaces;

      // insert the new text
      for (int j = 0; j < count; j++) {
        memmove(ptr, y_array[i].data, (size_t)yanklen);
        ptr += yanklen;

        // insert block's trailing spaces only if there's text behind
        if ((j < count - 1 || !shortline) && spaces > 0) {
          memset(ptr, ' ', (size_t)spaces);
          ptr += spaces;
        } else {
          totlen -= (size_t)spaces;  // didn't use these spaces
        }
      }

      // may insert some spaces after the new text
      memset(ptr, ' ', (size_t)bd.endspaces);
      ptr += bd.endspaces;

      // move the text after the cursor to the end of the line.
      int columns = oldlen - bd.textcol - delcount + 1;
      assert(columns >= 0);
      memmove(ptr, oldp + bd.textcol + delcount, (size_t)columns);
      ml_replace(curwin->w_cursor.lnum, newp, false);
      extmark_splice_cols(curbuf, (int)curwin->w_cursor.lnum - 1, bd.textcol,
                          delcount, (int)totlen + lines_appended, kExtmarkUndo);

      curwin->w_cursor.lnum++;
      if (i == 0) {
        curwin->w_cursor.col += bd.startspaces;
      }
    }

    changed_lines(curbuf, lnum, 0, curbuf->b_op_start.lnum + (linenr_T)y_size
                  - nr_lines, nr_lines, true);

    // Set '[ mark.
    curbuf->b_op_start = curwin->w_cursor;
    curbuf->b_op_start.lnum = lnum;

    // adjust '] mark
    curbuf->b_op_end.lnum = curwin->w_cursor.lnum - 1;
    curbuf->b_op_end.col = MAX(bd.textcol + (colnr_T)totlen - 1, 0);
    curbuf->b_op_end.coladd = 0;
    if (flags & PUT_CURSEND) {
      curwin->w_cursor = curbuf->b_op_end;
      curwin->w_cursor.col++;

      // in Insert mode we might be after the NUL, correct for that
      colnr_T len = get_cursor_line_len();
      curwin->w_cursor.col = MIN(curwin->w_cursor.col, len);
    } else {
      curwin->w_cursor.lnum = lnum;
    }
  } else {
    const int yanklen = (int)y_array[0].size;

    // Character or Line mode
    if (y_type == kMTCharWise) {
      // if type is kMTCharWise, FORWARD is the same as BACKWARD on the next
      // char
      if (dir == FORWARD && gchar_cursor() != NUL) {
        int bytelen = utfc_ptr2len(get_cursor_pos_ptr());

        // put it on the next of the multi-byte character.
        col += bytelen;
        if (yanklen) {
          curwin->w_cursor.col += bytelen;
          curbuf->b_op_end.col += bytelen;
        }
      }
      curbuf->b_op_start = curwin->w_cursor;
    } else if (dir == BACKWARD) {
      // Line mode: BACKWARD is the same as FORWARD on the previous line
      lnum--;
    }
    pos_T new_cursor = curwin->w_cursor;

    // simple case: insert into one line at a time
    if (y_type == kMTCharWise && y_size == 1) {
      linenr_T end_lnum = 0;  // init for gcc
      linenr_T start_lnum = lnum;
      int first_byte_off = 0;

      if (VIsual_active) {
        end_lnum = MAX(curbuf->b_visual.vi_end.lnum, curbuf->b_visual.vi_start.lnum);
        if (end_lnum > start_lnum) {
          // "col" is valid for the first line, in following lines
          // the virtual column needs to be used.  Matters for
          // multi-byte characters.
          pos_T pos = {
            .lnum = lnum,
            .col = col,
            .coladd = 0,
          };
          getvcol(curwin, &pos, NULL, &vcol, NULL);
        }
      }

      if (count == 0 || yanklen == 0) {
        if (VIsual_active) {
          lnum = end_lnum;
        }
      } else if (count > INT_MAX / yanklen) {
        // multiplication overflow
        emsg(_(e_resulting_text_too_long));
      } else {
        totlen = (size_t)count * (size_t)yanklen;
        do {
          char *oldp = ml_get(lnum);
          colnr_T oldlen = ml_get_len(lnum);
          if (lnum > start_lnum) {
            pos_T pos = {
              .lnum = lnum,
            };
            if (getvpos(curwin, &pos, vcol) == OK) {
              col = pos.col;
            } else {
              col = MAXCOL;
            }
          }
          if (VIsual_active && col > oldlen) {
            lnum++;
            continue;
          }
          char *newp = xmalloc(totlen + (size_t)oldlen + 1);
          memmove(newp, oldp, (size_t)col);
          char *ptr = newp + col;
          for (size_t i = 0; i < (size_t)count; i++) {
            memmove(ptr, y_array[0].data, (size_t)yanklen);
            ptr += yanklen;
          }
          memmove(ptr, oldp + col, (size_t)(oldlen - col) + 1);  // +1 for NUL
          ml_replace(lnum, newp, false);

          // compute the byte offset for the last character
          first_byte_off = utf_head_off(newp, ptr - 1);

          // Place cursor on last putted char.
          if (lnum == curwin->w_cursor.lnum) {
            // make sure curwin->w_virtcol is updated
            changed_cline_bef_curs(curwin);
            invalidate_botline(curwin);
            curwin->w_cursor.col += (colnr_T)(totlen - 1);
          }
          changed_bytes(lnum, col);
          extmark_splice_cols(curbuf, (int)lnum - 1, col,
                              0, (int)totlen, kExtmarkUndo);
          if (VIsual_active) {
            lnum++;
          }
        } while (VIsual_active && lnum <= end_lnum);

        if (VIsual_active) {  // reset lnum to the last visual line
          lnum--;
        }
      }

      // put '] at the first byte of the last character
      curbuf->b_op_end = curwin->w_cursor;
      curbuf->b_op_end.col -= first_byte_off;

      // For "CTRL-O p" in Insert mode, put cursor after last char
      if (totlen && (restart_edit != 0 || (flags & PUT_CURSEND))) {
        curwin->w_cursor.col++;
      } else {
        curwin->w_cursor.col -= first_byte_off;
      }
    } else {
      linenr_T new_lnum = new_cursor.lnum;
      int indent;
      int orig_indent = 0;
      int indent_diff = 0;        // init for gcc
      bool first_indent = true;
      int lendiff = 0;

      if (flags & PUT_FIXINDENT) {
        orig_indent = get_indent();
      }

      // Insert at least one line.  When y_type is kMTCharWise, break the first
      // line in two.
      for (int cnt = 1; cnt <= count; cnt++) {
        size_t i = 0;
        if (y_type == kMTCharWise) {
          // Split the current line in two at the insert position.
          // First insert y_array[size - 1] in front of second line.
          // Then append y_array[0] to first line.
          lnum = new_cursor.lnum;
          char *ptr = ml_get(lnum) + col;
          size_t ptrlen = (size_t)ml_get_len(lnum) - (size_t)col;
          totlen = y_array[y_size - 1].size;
          char *newp = xmalloc(ptrlen + totlen + 1);
          STRCPY(newp, y_array[y_size - 1].data);
          STRCPY(newp + totlen, ptr);
          // insert second line
          ml_append(lnum, newp, 0, false);
          new_lnum++;
          xfree(newp);

          char *oldp = ml_get(lnum);
          newp = xmalloc((size_t)col + (size_t)yanklen + 1);
          // copy first part of line
          memmove(newp, oldp, (size_t)col);
          // append to first line
          memmove(newp + col, y_array[0].data, (size_t)yanklen + 1);
          ml_replace(lnum, newp, false);

          curwin->w_cursor.lnum = lnum;
          i = 1;
        }

        for (; i < y_size; i++) {
          if ((y_type != kMTCharWise || i < y_size - 1)) {
            if (ml_append(lnum, y_array[i].data, 0, false) == FAIL) {
              goto error;
            }
            new_lnum++;
          }
          lnum++;
          nr_lines++;
          if (flags & PUT_FIXINDENT) {
            pos_T old_pos = curwin->w_cursor;
            curwin->w_cursor.lnum = lnum;
            char *ptr = ml_get(lnum);
            if (cnt == count && i == y_size - 1) {
              lendiff = ml_get_len(lnum);
            }
            if (*ptr == '#' && preprocs_left()) {
              indent = 0;                   // Leave # lines at start
            } else if (*ptr == NUL) {
              indent = 0;                   // Ignore empty lines
            } else if (first_indent) {
              indent_diff = orig_indent - get_indent();
              indent = orig_indent;
              first_indent = false;
            } else if ((indent = get_indent() + indent_diff) < 0) {
              indent = 0;
            }
            set_indent(indent, SIN_NOMARK);
            curwin->w_cursor = old_pos;
            // remember how many chars were removed
            if (cnt == count && i == y_size - 1) {
              lendiff -= ml_get_len(lnum);
            }
          }
        }

        bcount_t totsize = 0;
        int lastsize = 0;
        if (y_type == kMTCharWise
            || (y_type == kMTLineWise && (flags & PUT_LINE_SPLIT))) {
          for (i = 0; i < y_size - 1; i++) {
            totsize += (bcount_t)y_array[i].size + 1;
          }
          lastsize = (int)y_array[y_size - 1].size;
          totsize += lastsize;
        }
        if (y_type == kMTCharWise) {
          extmark_splice(curbuf, (int)new_cursor.lnum - 1, col, 0, 0, 0,
                         (int)y_size - 1, lastsize, totsize,
                         kExtmarkUndo);
        } else if (y_type == kMTLineWise && (flags & PUT_LINE_SPLIT)) {
          // Account for last pasted NL + last NL
          extmark_splice(curbuf, (int)new_cursor.lnum - 1, split_pos, 0, 0, 0,
                         (int)y_size + 1, 0, totsize + 2, kExtmarkUndo);
        }

        if (cnt == 1) {
          new_lnum = lnum;
        }
      }

error:
      // Adjust marks.
      if (y_type == kMTLineWise) {
        curbuf->b_op_start.col = 0;
        if (dir == FORWARD) {
          curbuf->b_op_start.lnum++;
        }
      }

      ExtmarkOp kind = (y_type == kMTLineWise && !(flags & PUT_LINE_SPLIT))
                       ? kExtmarkUndo : kExtmarkNOOP;
      mark_adjust(curbuf->b_op_start.lnum + (y_type == kMTCharWise),
                  (linenr_T)MAXLNUM, nr_lines, 0, kind);

      // note changed text for displaying and folding
      if (y_type == kMTCharWise) {
        changed_lines(curbuf, curwin->w_cursor.lnum, col,
                      curwin->w_cursor.lnum + 1, nr_lines, true);
      } else {
        changed_lines(curbuf, curbuf->b_op_start.lnum, 0,
                      curbuf->b_op_start.lnum, nr_lines, true);
      }

      // Put the '] mark on the first byte of the last inserted character.
      // Correct the length for change in indent.
      curbuf->b_op_end.lnum = new_lnum;
      col = MAX(0, (colnr_T)y_array[y_size - 1].size - lendiff);
      if (col > 1) {
        curbuf->b_op_end.col = col - 1;
        if (y_array[y_size - 1].size > 0) {
          curbuf->b_op_end.col -= utf_head_off(y_array[y_size - 1].data,
                                               y_array[y_size - 1].data
                                               + y_array[y_size - 1].size - 1);
        }
      } else {
        curbuf->b_op_end.col = 0;
      }

      if (flags & PUT_CURSLINE) {
        // ":put": put cursor on last inserted line
        curwin->w_cursor.lnum = lnum;
        beginline(BL_WHITE | BL_FIX);
      } else if (flags & PUT_CURSEND) {
        // put cursor after inserted text
        if (y_type == kMTLineWise) {
          if (lnum >= curbuf->b_ml.ml_line_count) {
            curwin->w_cursor.lnum = curbuf->b_ml.ml_line_count;
          } else {
            curwin->w_cursor.lnum = lnum + 1;
          }
          curwin->w_cursor.col = 0;
        } else {
          curwin->w_cursor.lnum = new_lnum;
          curwin->w_cursor.col = col;
          curbuf->b_op_end = curwin->w_cursor;
          if (col > 1) {
            curbuf->b_op_end.col = col - 1;
          }
        }
      } else if (y_type == kMTLineWise) {
        // put cursor on first non-blank in first inserted line
        curwin->w_cursor.col = 0;
        if (dir == FORWARD) {
          curwin->w_cursor.lnum++;
        }
        beginline(BL_WHITE | BL_FIX);
      } else {  // put cursor on first inserted character
        curwin->w_cursor = new_cursor;
      }
    }
  }

  msgmore(nr_lines);
  curwin->w_set_curswant = true;

  // Make sure the cursor is not after the NUL.
  int len = get_cursor_line_len();
  if (curwin->w_cursor.col > len) {
    if (cur_ve_flags == kOptVeFlagAll) {
      curwin->w_cursor.coladd = curwin->w_cursor.col - len;
    }
    curwin->w_cursor.col = len;
  }

end:
  if (cmdmod.cmod_flags & CMOD_LOCKMARKS) {
    curbuf->b_op_start = orig_start;
    curbuf->b_op_end = orig_end;
  }
  if (allocated) {
    xfree(insert_string.data);
  }
  if (regname == '=') {
    xfree(y_array);
  }

  VIsual_active = false;

  // If the cursor is past the end of the line put it at the end.
  adjust_cursor_eol();
}

// dis_msg and ex_display are implemented in Rust (src/nvim-rs/register/src/lib.rs).



