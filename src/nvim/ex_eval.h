#pragma once

#include <stdbool.h>

#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/ex_eval_defs.h"  // IWYU pragma: keep

#include "ex_eval.h.generated.h"

// Functions implemented in Rust (src/nvim-rs/ex_eval/)
bool aborting(void);
bool should_abort(int retcode);
bool aborted_in_try(void);
void update_force_abort(void);
void exception_state_save(exception_state_T *estate);
void exception_state_restore(exception_state_T *estate);
void exception_state_clear(void);
bool has_loop_cmd(char *p);
void ex_endfunction(exarg_T *eap);
void free_global_msglist(void);
void report_make_pending(int pending, void *value);
char *get_exception_string(void *value, except_type_T type, char *cmdname, bool *should_free);
int throw_exception(void *value, except_type_T type, char *cmdname);
void discard_exception(except_T *excp, bool was_finished);
void discard_current_exception(void);
void catch_exception(except_T *excp);
void finish_exception(except_T *excp);
void rewind_conditionals(cstack_T *cstack, int idx, int cond_type, int *cond_level);
bool cause_errthrow(const char *mesg, bool multiline, bool severe, bool *ignore);
void do_errthrow(cstack_T *cstack, char *cmdname);
bool do_intthrow(cstack_T *cstack);
void do_throw(cstack_T *cstack);
void report_pending(int action, int pending, void *value);
void report_resume_pending(int pending, void *value);
void report_discard_pending(int pending, void *value);
void enter_cleanup(cleanup_T *csp);
void leave_cleanup(cleanup_T *csp);
