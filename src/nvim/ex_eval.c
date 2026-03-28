/// @file ex_eval.c
///
/// Functions for Ex command line for the +eval feature.
#include <assert.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/charset.h"
#include "nvim/debugger.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/eval_defs.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_eval_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option_vars.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/runtime.h"
#include "nvim/runtime_defs.h"
#include "nvim/strings.h"
#include "nvim/vim_defs.h"

#include "ex_eval.c.generated.h"

static const char e_multiple_else[] = N_("E583: Multiple :else");
static const char e_multiple_finally[] = N_("E607: Multiple :finally");

// Exception handling terms:
//
//      :try            ":try" command         ─┐
//          ...         try block               │
//      :catch RE       ":catch" command        │
//          ...         catch clause            ├─ try conditional
//      :finally        ":finally" command      │
//          ...         finally clause          │
//      :endtry         ":endtry" command      ─┘
//
// The try conditional may have any number of catch clauses and at most one
// finally clause.  A ":throw" command can be inside the try block, a catch
// clause, the finally clause, or in a function called or script sourced from
// there or even outside the try conditional.  Try conditionals may be nested.

// Configuration whether an exception is thrown on error or interrupt.  When
// the preprocessor macros below evaluate to false, an error (did_emsg) or
// interrupt (got_int) under an active try conditional terminates the script
// after the non-active finally clauses of all active try conditionals have been
// executed.  Otherwise, errors and/or interrupts are converted into catchable
// exceptions (did_throw additionally set), which terminate the script only if
// not caught.  For user exceptions, only did_throw is set.  (Note: got_int can
// be set asynchronously afterwards by a SIGINT, so did_throw && got_int is not
// a reliant test that the exception currently being thrown is an interrupt
// exception.  Similarly, did_emsg can be set afterwards on an error in an
// (unskipped) conditional command inside an inactive conditional, so did_throw
// && did_emsg is not a reliant test that the exception currently being thrown
// is an error exception.)  -  The macros can be defined as expressions checking
// for a variable that is allowed to be changed during execution of a script.

// Values used for the Vim release.
#define THROW_ON_ERROR true
#define THROW_ON_ERROR_TRUE
#define THROW_ON_INTERRUPT true
#define THROW_ON_INTERRUPT_TRUE

// Don't do something after an error, interrupt, or throw, or when
// there is a surrounding conditional and it was not active.
#define CHECK_SKIP \
  (did_emsg \
   || got_int \
   || did_throw \
   || (cstack->cs_idx > 0 \
       && !(cstack->cs_flags[cstack->cs_idx - 1] & CSF_ACTIVE)))

// cause_abort is now owned by Rust (rs_get_cause_abort/rs_set_cause_abort).
// When several errors appear in a row, setting "force_abort" is delayed until
// the failing command returned.  "cause_abort" is set to true meanwhile, in
// order to indicate that situation.  This is useful when "force_abort" was set
// during execution of a function call from an expression: the aborting of the
// expression evaluation is done without producing any error messages, but all
// error messages on parsing errors during the expression evaluation are given
// (even if a try conditional is active).
extern bool rs_get_cause_abort(void);
extern void rs_set_cause_abort(bool val);
// free_msglist is now implemented in Rust
extern void free_msglist(msglist_T *l);
// discard_pending_return is now implemented in Rust
extern void discard_pending_return(typval_T *p);
/// Handle ":else" and ":elseif".
void ex_else(exarg_T *eap)
{
  cstack_T *const cstack = eap->cstack;

  bool skip = CHECK_SKIP;

  if (cstack->cs_idx < 0
      || (cstack->cs_flags[cstack->cs_idx]
          & (CSF_WHILE | CSF_FOR | CSF_TRY))) {
    if (eap->cmdidx == CMD_else) {
      eap->errmsg = _("E581: :else without :if");
      return;
    }
    eap->errmsg = _("E582: :elseif without :if");
    skip = true;
  } else if (cstack->cs_flags[cstack->cs_idx] & CSF_ELSE) {
    if (eap->cmdidx == CMD_else) {
      eap->errmsg = _(e_multiple_else);
      return;
    }
    eap->errmsg = _("E584: :elseif after :else");
    skip = true;
  }

  // if skipping or the ":if" was TRUE, reset ACTIVE, otherwise set it
  if (skip || cstack->cs_flags[cstack->cs_idx] & CSF_TRUE) {
    if (eap->errmsg == NULL) {
      cstack->cs_flags[cstack->cs_idx] = CSF_TRUE;
    }
    skip = true;        // don't evaluate an ":elseif"
  } else {
    cstack->cs_flags[cstack->cs_idx] = CSF_ACTIVE;
  }

  // When debugging or a breakpoint was encountered, display the debug prompt
  // (if not already done).  This shows the user that an ":else" or ":elseif"
  // is executed when the ":if" or previous ":elseif" was not TRUE.  Handle
  // a ">quit" debug command as if an interrupt had occurred before the
  // ":else" or ":elseif".  That is, set "skip" and throw an interrupt
  // exception if appropriate.  Doing this here prevents that an exception
  // for a parsing errors is discarded when throwing the interrupt exception
  // later on.
  if (!skip && dbg_check_skipped(eap) && got_int) {
    do_intthrow(cstack);
    skip = true;
  }

  if (eap->cmdidx == CMD_elseif) {
    bool result = false;
    bool error;
    // When skipping we ignore most errors, but a missing expression is
    // wrong, perhaps it should have been "else".
    // A double quote here is the start of a string, not a comment.
    if (skip && *eap->arg != '"' && ends_excmd(*eap->arg)) {
      semsg(_(e_invexpr2), eap->arg);
    } else {
      result = eval_to_bool(eap->arg, &error, eap, skip, false);
    }

    // When throwing error exceptions, we want to throw always the first
    // of several errors in a row.  This is what actually happens when
    // a conditional error was detected above and there is another failure
    // when parsing the expression.  Since the skip flag is set in this
    // case, the parsing error will be ignored by emsg().
    if (!skip && !error) {
      if (result) {
        cstack->cs_flags[cstack->cs_idx] = CSF_ACTIVE | CSF_TRUE;
      } else {
        cstack->cs_flags[cstack->cs_idx] = 0;
      }
    } else if (eap->errmsg == NULL) {
      // set TRUE, so this conditional will never get active
      cstack->cs_flags[cstack->cs_idx] = CSF_TRUE;
    }
  } else {
    cstack->cs_flags[cstack->cs_idx] |= CSF_ELSE;
  }
}

/// Handle ":while" and ":for".
void ex_while(exarg_T *eap)
{
  bool error;
  cstack_T *const cstack = eap->cstack;

  if (cstack->cs_idx == CSTACK_LEN - 1) {
    eap->errmsg = _("E585: :while/:for nesting too deep");
  } else {
    bool result;
    // The loop flag is set when we have jumped back from the matching
    // ":endwhile" or ":endfor".  When not set, need to initialise this
    // cstack entry.
    if ((cstack->cs_lflags & CSL_HAD_LOOP) == 0) {
      cstack->cs_idx++;
      cstack->cs_looplevel++;
      cstack->cs_line[cstack->cs_idx] = -1;
    }
    cstack->cs_flags[cstack->cs_idx] =
      eap->cmdidx == CMD_while ? CSF_WHILE : CSF_FOR;

    int skip = CHECK_SKIP;
    if (eap->cmdidx == CMD_while) {  // ":while bool-expr"
      result = eval_to_bool(eap->arg, &error, eap, skip, false);
    } else {  // ":for var in list-expr"
      evalarg_T evalarg;
      fill_evalarg_from_eap(&evalarg, eap, skip);
      void *fi;
      if ((cstack->cs_lflags & CSL_HAD_LOOP) != 0) {
        // Jumping here from a ":continue" or ":endfor": use the
        // previously evaluated list.
        fi = cstack->cs_forinfo[cstack->cs_idx];
        error = false;
      } else {
        // Evaluate the argument and get the info in a structure.
        fi = eval_for_line(eap->arg, &error, eap, &evalarg);
        cstack->cs_forinfo[cstack->cs_idx] = fi;
      }

      // use the element at the start of the list and advance
      if (!error && fi != NULL && !skip) {
        result = next_for_item(fi, eap->arg);
      } else {
        result = false;
      }

      if (!result) {
        free_for_info(fi);
        cstack->cs_forinfo[cstack->cs_idx] = NULL;
      }
      clear_evalarg(&evalarg, eap);
    }

    // If this cstack entry was just initialised and is active, set the
    // loop flag, so do_cmdline() will set the line number in cs_line[].
    // If executing the command a second time, clear the loop flag.
    if (!skip && !error && result) {
      cstack->cs_flags[cstack->cs_idx] |= (CSF_ACTIVE | CSF_TRUE);
      cstack->cs_lflags ^= CSL_HAD_LOOP;
    } else {
      cstack->cs_lflags &= ~CSL_HAD_LOOP;
      // If the ":while" evaluates to FALSE or ":for" is past the end of
      // the list, show the debug prompt at the ":endwhile"/":endfor" as
      // if there was a ":break" in a ":while"/":for" evaluating to
      // TRUE.
      if (!skip && !error) {
        cstack->cs_flags[cstack->cs_idx] |= CSF_TRUE;
      }
    }
  }
}

/// Handle ":endwhile" and ":endfor"
void ex_endwhile(exarg_T *eap)
{
  cstack_T *const cstack = eap->cstack;
  const char *err;
  int csf;

  if (eap->cmdidx == CMD_endwhile) {
    err = e_while;
    csf = CSF_WHILE;
  } else {
    err = e_for;
    csf = CSF_FOR;
  }

  if (cstack->cs_looplevel <= 0 || cstack->cs_idx < 0) {
    eap->errmsg = _(err);
  } else {
    int fl = cstack->cs_flags[cstack->cs_idx];
    if (!(fl & csf)) {
      // If we are in a ":while" or ":for" but used the wrong endloop
      // command, do not rewind to the next enclosing ":for"/":while".
      if (fl & CSF_WHILE) {
        eap->errmsg = _("E732: Using :endfor with :while");
      } else if (fl & CSF_FOR) {
        eap->errmsg = _("E733: Using :endwhile with :for");
      }
    }
    if (!(fl & (CSF_WHILE | CSF_FOR))) {
      if (!(fl & CSF_TRY)) {
        eap->errmsg = _(e_endif);
      } else if (fl & CSF_FINALLY) {
        eap->errmsg = _(e_endtry);
      }
      // Try to find the matching ":while" and report what's missing.
      int idx;
      for (idx = cstack->cs_idx; idx > 0; idx--) {
        fl = cstack->cs_flags[idx];
        if ((fl & CSF_TRY) && !(fl & CSF_FINALLY)) {
          // Give up at a try conditional not in its finally clause.
          // Ignore the ":endwhile"/":endfor".
          eap->errmsg = _(err);
          return;
        }
        if (fl & csf) {
          break;
        }
      }
      // Cleanup and rewind all contained (and unclosed) conditionals.
      cleanup_conditionals(cstack, CSF_WHILE | CSF_FOR, false);
      rewind_conditionals(cstack, idx, CSF_TRY, &cstack->cs_trylevel);
    } else if (cstack->cs_flags[cstack->cs_idx] & CSF_TRUE
               && !(cstack->cs_flags[cstack->cs_idx] & CSF_ACTIVE)
               && dbg_check_skipped(eap)) {
      // When debugging or a breakpoint was encountered, display the debug
      // prompt (if not already done).  This shows the user that an
      // ":endwhile"/":endfor" is executed when the ":while" was not TRUE or
      // after a ":break".  Handle a ">quit" debug command as if an
      // interrupt had occurred before the ":endwhile"/":endfor".  That is,
      // throw an interrupt exception if appropriate.  Doing this here
      // prevents that an exception for a parsing error is discarded when
      // throwing the interrupt exception later on.
      do_intthrow(cstack);
    }

    // Set loop flag, so do_cmdline() will jump back to the matching
    // ":while" or ":for".
    cstack->cs_lflags |= CSL_HAD_ENDLOOP;
  }
}

/// Handle ":catch /{pattern}/" and ":catch"
void ex_catch(exarg_T *eap)
{
  int idx = 0;
  bool give_up = false;
  bool skip = false;
  char *end;
  char *save_cpo;
  regmatch_T regmatch;
  cstack_T *const cstack = eap->cstack;
  char *pat;

  if (cstack->cs_trylevel <= 0 || cstack->cs_idx < 0) {
    eap->errmsg = _("E603: :catch without :try");
    give_up = true;
  } else {
    if (!(cstack->cs_flags[cstack->cs_idx] & CSF_TRY)) {
      // Report what's missing if the matching ":try" is not in its
      // finally clause.
      eap->errmsg = get_end_emsg(cstack);
      skip = true;
    }
    for (idx = cstack->cs_idx; idx > 0; idx--) {
      if (cstack->cs_flags[idx] & CSF_TRY) {
        break;
      }
    }
    if (cstack->cs_flags[idx] & CSF_FINALLY) {
      // Give up for a ":catch" after ":finally" and ignore it.
      // Just parse.
      eap->errmsg = _("E604: :catch after :finally");
      give_up = true;
    } else {
      rewind_conditionals(cstack, idx, CSF_WHILE | CSF_FOR,
                          &cstack->cs_looplevel);
    }
  }

  if (ends_excmd(*eap->arg)) {  // no argument, catch all errors
    pat = ".*";
    end = NULL;
    eap->nextcmd = find_nextcmd(eap->arg);
  } else {
    pat = eap->arg + 1;
    end = skip_regexp_err(pat, *eap->arg, true);
    if (end == NULL) {
      give_up = true;
    }
  }

  if (!give_up) {
    bool caught = false;
    // Don't do something when no exception has been thrown or when the
    // corresponding try block never got active (because of an inactive
    // surrounding conditional or after an error or interrupt or throw).
    if (!did_throw || !(cstack->cs_flags[idx] & CSF_TRUE)) {
      skip = true;
    }

    // Check for a match only if an exception is thrown but not caught by
    // a previous ":catch".  An exception that has replaced a discarded
    // exception is not checked (THROWN is not set then).
    if (!skip && (cstack->cs_flags[idx] & CSF_THROWN)
        && !(cstack->cs_flags[idx] & CSF_CAUGHT)) {
      if (end != NULL && *end != NUL && !ends_excmd(*skipwhite(end + 1))) {
        semsg(_(e_trailing_arg), end);
        return;
      }

      // When debugging or a breakpoint was encountered, display the
      // debug prompt (if not already done) before checking for a match.
      // This is a helpful hint for the user when the regular expression
      // matching fails.  Handle a ">quit" debug command as if an
      // interrupt had occurred before the ":catch".  That is, discard
      // the original exception, replace it by an interrupt exception,
      // and don't catch it in this try block.
      if (!dbg_check_skipped(eap) || !do_intthrow(cstack)) {
        char save_char = 0;
        // Terminate the pattern and avoid the 'l' flag in 'cpoptions'
        // while compiling it.
        if (end != NULL) {
          save_char = *end;
          *end = NUL;
        }
        save_cpo = p_cpo;
        p_cpo = empty_string_option;
        // Disable error messages, it will make current exception
        // invalid
        emsg_off++;
        regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
        emsg_off--;
        regmatch.rm_ic = false;
        if (end != NULL) {
          *end = save_char;
        }
        p_cpo = save_cpo;
        if (regmatch.regprog == NULL) {
          semsg(_(e_invarg2), pat);
        } else {
          // Save the value of got_int and reset it.  We don't want
          // a previous interruption cancel matching, only hitting
          // CTRL-C while matching should abort it.

          int prev_got_int = got_int;
          got_int = false;
          caught = vim_regexec_nl(&regmatch, current_exception->value, 0);
          got_int |= prev_got_int;
          vim_regfree(regmatch.regprog);
        }
      }
    }

    if (caught) {
      // Make this ":catch" clause active and reset did_emsg, got_int,
      // and did_throw.  Put the exception on the caught stack.
      cstack->cs_flags[idx] |= CSF_ACTIVE | CSF_CAUGHT;
      did_emsg = got_int = did_throw = false;
      catch_exception((except_T *)cstack->cs_exception[idx]);
      // It's mandatory that the current exception is stored in the cstack
      // so that it can be discarded at the next ":catch", ":finally", or
      // ":endtry" or when the catch clause is left by a ":continue",
      // ":break", ":return", ":finish", error, interrupt, or another
      // exception.
      if (cstack->cs_exception[cstack->cs_idx] != current_exception) {
        internal_error("ex_catch()");
      }
    } else {
      // If there is a preceding catch clause and it caught the exception,
      // finish the exception now.  This happens also after errors except
      // when this ":catch" was after the ":finally" or not within
      // a ":try".  Make the try conditional inactive so that the
      // following catch clauses are skipped.  On an error or interrupt
      // after the preceding try block or catch clause was left by
      // a ":continue", ":break", ":return", or ":finish", discard the
      // pending action.
      cleanup_conditionals(cstack, CSF_TRY, true);
    }
  }

  if (end != NULL) {
    eap->nextcmd = find_nextcmd(end);
  }
}

/// Handle ":finally"
void ex_finally(exarg_T *eap)
{
  int idx;
  int pending = CSTP_NONE;
  cstack_T *const cstack = eap->cstack;

  for (idx = cstack->cs_idx; idx >= 0; idx--) {
    if (cstack->cs_flags[idx] & CSF_TRY) {
      break;
    }
  }
  if (cstack->cs_trylevel <= 0 || idx < 0) {
    eap->errmsg = _("E606: :finally without :try");
    return;
  }

  if (!(cstack->cs_flags[cstack->cs_idx] & CSF_TRY)) {
    eap->errmsg = get_end_emsg(cstack);
    // Make this error pending, so that the commands in the following
    // finally clause can be executed.  This overrules also a pending
    // ":continue", ":break", ":return", or ":finish".
    pending = CSTP_ERROR;
  }

  if (cstack->cs_flags[idx] & CSF_FINALLY) {
    // Give up for a multiple ":finally" and ignore it.
    eap->errmsg = _(e_multiple_finally);
    return;
  }
  rewind_conditionals(cstack, idx, CSF_WHILE | CSF_FOR,
                      &cstack->cs_looplevel);

  // Don't do something when the corresponding try block never got active
  // (because of an inactive surrounding conditional or after an error or
  // interrupt or throw) or for a ":finally" without ":try" or a multiple
  // ":finally".  After every other error (did_emsg or the conditional
  // errors detected above) or after an interrupt (got_int) or an
  // exception (did_throw), the finally clause must be executed.
  int skip = !(cstack->cs_flags[cstack->cs_idx] & CSF_TRUE);

  if (!skip) {
    // When debugging or a breakpoint was encountered, display the
    // debug prompt (if not already done).  The user then knows that the
    // finally clause is executed.
    if (dbg_check_skipped(eap)) {
      // Handle a ">quit" debug command as if an interrupt had
      // occurred before the ":finally".  That is, discard the
      // original exception and replace it by an interrupt
      // exception.
      do_intthrow(cstack);
    }

    // If there is a preceding catch clause and it caught the exception,
    // finish the exception now.  This happens also after errors except
    // when this is a multiple ":finally" or one not within a ":try".
    // After an error or interrupt, this also discards a pending
    // ":continue", ":break", ":finish", or ":return" from the preceding
    // try block or catch clause.
    cleanup_conditionals(cstack, CSF_TRY, false);

    // Make did_emsg, got_int, did_throw pending.  If set, they overrule
    // a pending ":continue", ":break", ":return", or ":finish".  Then
    // we have particularly to discard a pending return value (as done
    // by the call to cleanup_conditionals() above when did_emsg or
    // got_int is set).  The pending values are restored by the
    // ":endtry", except if there is a new error, interrupt, exception,
    // ":continue", ":break", ":return", or ":finish" in the following
    // finally clause.  A missing ":endwhile", ":endfor" or ":endif"
    // detected here is treated as if did_emsg and did_throw had
    // already been set, respectively in case that the error is not
    // converted to an exception, did_throw had already been unset.
    // We must not set did_emsg here since that would suppress the
    // error message.
    if (pending == CSTP_ERROR || did_emsg || got_int || did_throw) {
      if (cstack->cs_pending[cstack->cs_idx] == CSTP_RETURN) {
        report_discard_pending(CSTP_RETURN,
                               cstack->cs_rettv[cstack->cs_idx]);
        discard_pending_return(cstack->cs_rettv[cstack->cs_idx]);
      }
      if (pending == CSTP_ERROR && !did_emsg) {
        pending |= (THROW_ON_ERROR ? CSTP_THROW : 0);
      } else {
        pending |= (did_throw ? CSTP_THROW : 0);
      }
      pending |= did_emsg ? CSTP_ERROR : 0;
      pending |= got_int ? CSTP_INTERRUPT : 0;
      assert(pending >= CHAR_MIN && pending <= CHAR_MAX);
      cstack->cs_pending[cstack->cs_idx] = (char)pending;

      // It's mandatory that the current exception is stored in the
      // cstack so that it can be rethrown at the ":endtry" or be
      // discarded if the finally clause is left by a ":continue",
      // ":break", ":return", ":finish", error, interrupt, or another
      // exception.  When emsg() is called for a missing ":endif" or
      // a missing ":endwhile"/":endfor" detected here, the
      // exception will be discarded.
      if (did_throw && cstack->cs_exception[cstack->cs_idx] != current_exception) {
        internal_error("ex_finally()");
      }
    }

    // Set CSL_HAD_FINA, so do_cmdline() will reset did_emsg,
    // got_int, and did_throw and make the finally clause active.
    // This will happen after emsg() has been called for a missing
    // ":endif" or a missing ":endwhile"/":endfor" detected here, so
    // that the following finally clause will be executed even then.
    cstack->cs_lflags |= CSL_HAD_FINA;
  }
}

/// Handle ":endtry"
void ex_endtry(exarg_T *eap)
{
  int idx;
  bool rethrow = false;
  char pending = CSTP_NONE;
  void *rettv = NULL;
  cstack_T *const cstack = eap->cstack;

  for (idx = cstack->cs_idx; idx >= 0; idx--) {
    if (cstack->cs_flags[idx] & CSF_TRY) {
      break;
    }
  }
  if (cstack->cs_trylevel <= 0 || idx < 0) {
    eap->errmsg = _("E602: :endtry without :try");
    return;
  }

  // Don't do something after an error, interrupt or throw in the try
  // block, catch clause, or finally clause preceding this ":endtry" or
  // when an error or interrupt occurred after a ":continue", ":break",
  // ":return", or ":finish" in a try block or catch clause preceding this
  // ":endtry" or when the try block never got active (because of an
  // inactive surrounding conditional or after an error or interrupt or
  // throw) or when there is a surrounding conditional and it has been
  // made inactive by a ":continue", ":break", ":return", or ":finish" in
  // the finally clause.  The latter case need not be tested since then
  // anything pending has already been discarded.
  bool skip = did_emsg || got_int || did_throw || !(cstack->cs_flags[cstack->cs_idx] & CSF_TRUE);

  if (!(cstack->cs_flags[cstack->cs_idx] & CSF_TRY)) {
    eap->errmsg = get_end_emsg(cstack);

    // Find the matching ":try" and report what's missing.
    rewind_conditionals(cstack, idx, CSF_WHILE | CSF_FOR,
                        &cstack->cs_looplevel);
    skip = true;

    // If an exception is being thrown, discard it to prevent it from
    // being rethrown at the end of this function.  It would be
    // discarded by the error message, anyway.  Resets did_throw.
    // This does not affect the script termination due to the error
    // since "trylevel" is decremented after emsg() has been called.
    if (did_throw) {
      discard_current_exception();
    }

    // report eap->errmsg, also when there already was an error
    did_emsg = false;
  } else {
    idx = cstack->cs_idx;

    // If we stopped with the exception currently being thrown at this
    // try conditional since we didn't know that it doesn't have
    // a finally clause, we need to rethrow it after closing the try
    // conditional.
    if (did_throw
        && (cstack->cs_flags[idx] & CSF_TRUE)
        && !(cstack->cs_flags[idx] & CSF_FINALLY)) {
      rethrow = true;
    }
  }

  // If there was no finally clause, show the user when debugging or
  // a breakpoint was encountered that the end of the try conditional has
  // been reached: display the debug prompt (if not already done).  Do
  // this on normal control flow or when an exception was thrown, but not
  // on an interrupt or error not converted to an exception or when
  // a ":break", ":continue", ":return", or ":finish" is pending.  These
  // actions are carried out immediately.
  if ((rethrow || (!skip
                   && !(cstack->cs_flags[idx] & CSF_FINALLY)
                   && !cstack->cs_pending[idx]))
      && dbg_check_skipped(eap)) {
    // Handle a ">quit" debug command as if an interrupt had occurred
    // before the ":endtry".  That is, throw an interrupt exception and
    // set "skip" and "rethrow".
    if (got_int) {
      skip = true;
      do_intthrow(cstack);
      // The do_intthrow() call may have reset did_throw or
      // cstack->cs_pending[idx].
      rethrow = false;
      if (did_throw && !(cstack->cs_flags[idx] & CSF_FINALLY)) {
        rethrow = true;
      }
    }
  }

  // If a ":return" is pending, we need to resume it after closing the
  // try conditional; remember the return value.  If there was a finally
  // clause making an exception pending, we need to rethrow it.  Make it
  // the exception currently being thrown.
  if (!skip) {
    pending = cstack->cs_pending[idx];
    cstack->cs_pending[idx] = CSTP_NONE;
    if (pending == CSTP_RETURN) {
      rettv = cstack->cs_rettv[idx];
    } else if (pending & CSTP_THROW) {
      current_exception = cstack->cs_exception[idx];
    }
  }

  // Discard anything pending on an error, interrupt, or throw in the
  // finally clause.  If there was no ":finally", discard a pending
  // ":continue", ":break", ":return", or ":finish" if an error or
  // interrupt occurred afterwards, but before the ":endtry" was reached.
  // If an exception was caught by the last of the catch clauses and there
  // was no finally clause, finish the exception now.  This happens also
  // after errors except when this ":endtry" is not within a ":try".
  // Restore "emsg_silent" if it has been reset by this try conditional.
  cleanup_conditionals(cstack, CSF_TRY | CSF_SILENT, true);

  if (cstack->cs_idx >= 0 && (cstack->cs_flags[cstack->cs_idx] & CSF_TRY)) {
    cstack->cs_idx--;
  }
  cstack->cs_trylevel--;

  if (!skip) {
    report_resume_pending(pending,
                          (pending == CSTP_RETURN)
                          ? rettv
                          : (pending & CSTP_THROW) ? (void *)current_exception : NULL);
    switch (pending) {
    case CSTP_NONE:
      break;

    // Reactivate a pending ":continue", ":break", ":return",
    // ":finish" from the try block or a catch clause of this try
    // conditional.  This is skipped, if there was an error in an
    // (unskipped) conditional command or an interrupt afterwards
    // or if the finally clause is present and executed a new error,
    // interrupt, throw, ":continue", ":break", ":return", or
    // ":finish".
    case CSTP_CONTINUE:
      ex_continue(eap);
      break;
    case CSTP_BREAK:
      ex_break(eap);
      break;
    case CSTP_RETURN:
      do_return(eap, false, false, rettv);
      break;
    case CSTP_FINISH:
      do_finish(eap, false);
      break;

    // When the finally clause was entered due to an error,
    // interrupt or throw (as opposed to a ":continue", ":break",
    // ":return", or ":finish"), restore the pending values of
    // did_emsg, got_int, and did_throw.  This is skipped, if there
    // was a new error, interrupt, throw, ":continue", ":break",
    // ":return", or ":finish".  in the finally clause.
    default:
      if (pending & CSTP_ERROR) {
        did_emsg = true;
      }
      if (pending & CSTP_INTERRUPT) {
        got_int = true;
      }
      if (pending & CSTP_THROW) {
        rethrow = true;
      }
      break;
    }
  }

  if (rethrow) {
    // Rethrow the current exception (within this cstack).
    do_throw(cstack);
  }
}

// cleanup_conditionals and get_end_emsg are now implemented in Rust
extern int cleanup_conditionals(cstack_T *cstack, int searched_cond, int inclusive);
extern char *get_end_emsg(cstack_T *cstack);


