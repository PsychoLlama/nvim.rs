#pragma once

#include <stddef.h>  // IWYU pragma: keep
#include <stdint.h>  // IWYU pragma: keep

#include "nvim/api/private/defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/getchar_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

/// Argument for flush_buffers().
typedef enum {
  FLUSH_MINIMAL,
  FLUSH_TYPEAHEAD,  ///< flush current typebuf contents
  FLUSH_INPUT,      ///< flush typebuf and inchar() input
} flush_buffers_T;

enum { NSCRIPT = 15, };  ///< Maximum number of streams to read script from

EXTERN bool test_disable_char_avail INIT( = false);

// Functions now implemented in Rust (via #[export_name]) -- Phase 1
#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>
#include "nvim/getchar_defs.h"
#include "nvim/undo_defs.h"

// Stuff buffer (readbuf1/readbuf2)
void stuffReadbuff(const char *s);
void stuffReadbuffLen(const char *s, ptrdiff_t len);
void stuffRedoReadbuff(const char *s);
void stuffcharReadbuff(int c);
void stuffnumReadbuff(int n);
void stuffReadbuffSpec(const char *s);
void stuffescaped(const char *arg, bool literally);
bool stuff_empty(void);
bool readbuf1_empty(void);

// Inserted text
String get_inserted(void);
void paste_store(uint64_t channel_id, TriState state, String str, bool crlf);

// Redo buffer
void AppendToRedobuff(const char *s);
void AppendToRedobuffLit(const char *str, int len);
void AppendToRedobuffSpec(const char *s);
void AppendCharToRedobuff(int c);
void AppendNumberToRedobuff(int n);
void ResetRedobuff(void);
void CancelRedo(void);
void saveRedobuff(save_redo_T *save_redo);
void restoreRedobuff(save_redo_T *save_redo);
int start_redo(int count, bool old_redo);
int start_redo_ins(void);
void stop_redo_ins(void);
char *get_recorded(void);

// Typeahead / buffer flushing
void flush_buffers(flush_buffers_T flush_typeahead);
void save_typeahead(tasave_T *tp);
void restore_typeahead(tasave_T *tp);

// Orchestrator functions (implemented in Rust, src/nvim-rs/getchar/src/orchestrator.rs)
void before_blocking(void);
int vgetc(void);
int safe_vgetc(void);
int plain_vgetc(void);
int vpeekc(void);
int vpeekc_any(void);
bool char_avail(void);
char *getcmdkeycmd(int promptc, void *cookie, int indent, bool do_concat);

// Internal typeahead helper (used by Rust orchestrators)
int vgetorpeek(bool advance);

// Typeahead input state
int ins_typebuf(char *str, int noremap, int offset, bool nottyped, bool silent);
void del_typebuf(int len, int offset);
void typeahead_noflush(int c);
void beep_flush(void);
void vungetc(int c);
int ins_char_typebuf(int c, int modifiers, bool on_key_ignore);
bool typebuf_changed(int tb_change_cnt);
int typebuf_typed(void);
int typebuf_maplen(void);
bool noremap_keys(void);
int merge_modifiers(int c_arg, int *modifiers);
int fix_input_buffer(uint8_t *buf, int len);
int using_script(void);
void may_sync_undo(void);
void gotchars_ignore(void);
void ungetchars(int len);
void check_end_reg_executing(bool advance);

#include "getchar.h.generated.h"
