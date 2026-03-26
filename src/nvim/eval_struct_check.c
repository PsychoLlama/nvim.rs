/// Static assertions validating that Rust eval struct mirrors match C layouts.
///
/// These compile-time checks ensure every type and field accessed directly from
/// Rust has the correct size, offset, and constant values.

#include <stddef.h>
#include <stdint.h>

#include "nvim/channel_defs.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval_defs.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/runtime_defs.h"

// --- Constant value assertions ---

_Static_assert(VARNUMBER_MAX == INT64_MAX, "VARNUMBER_MAX mismatch");
_Static_assert(FNE_INCL_BR == 1, "FNE_INCL_BR mismatch");
_Static_assert(FNE_CHECK_START == 2, "FNE_CHECK_START mismatch");
_Static_assert(RE_MAGIC == 1, "RE_MAGIC mismatch");
_Static_assert(RE_STRING == 2, "RE_STRING mismatch");
_Static_assert(VAR_NUMBER == 1, "VAR_NUMBER mismatch");
_Static_assert(VAR_STRING == 2, "VAR_STRING mismatch");
_Static_assert(VAR_FUNC == 3, "VAR_FUNC mismatch");
_Static_assert(VAR_SPECIAL == 8, "VAR_SPECIAL mismatch");
_Static_assert(VAR_PARTIAL == 9, "VAR_PARTIAL mismatch");
_Static_assert(VAR_DICT == 5, "VAR_DICT mismatch");
_Static_assert(VAR_LIST == 4, "VAR_LIST mismatch");
_Static_assert(kCallbackNone == 0, "kCallbackNone mismatch");
_Static_assert(kCallbackFuncref == 1, "kCallbackFuncref mismatch");
_Static_assert(kCallbackPartial == 2, "kCallbackPartial mismatch");

// --- lval_T layout assertions (Rust LvalT must match exactly) ---
_Static_assert(sizeof(lval_T) == 96, "lval_T size mismatch: Rust LvalT must be updated");
_Static_assert(offsetof(lval_T, ll_name) == 0, "lval_T ll_name offset mismatch");
_Static_assert(offsetof(lval_T, ll_name_len) == 8, "lval_T ll_name_len offset mismatch");
_Static_assert(offsetof(lval_T, ll_exp_name) == 16, "lval_T ll_exp_name offset mismatch");
_Static_assert(offsetof(lval_T, ll_tv) == 24, "lval_T ll_tv offset mismatch");
_Static_assert(offsetof(lval_T, ll_li) == 32, "lval_T ll_li offset mismatch");
_Static_assert(offsetof(lval_T, ll_list) == 40, "lval_T ll_list offset mismatch");
_Static_assert(offsetof(lval_T, ll_range) == 48, "lval_T ll_range offset mismatch");
_Static_assert(offsetof(lval_T, ll_empty2) == 49, "lval_T ll_empty2 offset mismatch");
_Static_assert(offsetof(lval_T, ll_n1) == 52, "lval_T ll_n1 offset mismatch");
_Static_assert(offsetof(lval_T, ll_n2) == 56, "lval_T ll_n2 offset mismatch");
_Static_assert(offsetof(lval_T, ll_dict) == 64, "lval_T ll_dict offset mismatch");
_Static_assert(offsetof(lval_T, ll_di) == 72, "lval_T ll_di offset mismatch");
_Static_assert(offsetof(lval_T, ll_newkey) == 80, "lval_T ll_newkey offset mismatch");
_Static_assert(offsetof(lval_T, ll_blob) == 88, "lval_T ll_blob offset mismatch");

// --- funcexe_T layout assertions (Rust FuncExeT must match exactly) ---
_Static_assert(sizeof(funcexe_T) == 64, "funcexe_T size mismatch: Rust FuncExeT must be updated");
_Static_assert(offsetof(funcexe_T, fe_argv_func) == 0, "funcexe_T fe_argv_func offset mismatch");
_Static_assert(offsetof(funcexe_T, fe_firstline) == 8, "funcexe_T fe_firstline offset mismatch");
_Static_assert(offsetof(funcexe_T, fe_lastline) == 12, "funcexe_T fe_lastline offset mismatch");
_Static_assert(offsetof(funcexe_T, fe_doesrange) == 16, "funcexe_T fe_doesrange offset mismatch");
_Static_assert(offsetof(funcexe_T, fe_evaluate) == 24, "funcexe_T fe_evaluate offset mismatch");
_Static_assert(offsetof(funcexe_T, fe_partial) == 32, "funcexe_T fe_partial offset mismatch");
_Static_assert(offsetof(funcexe_T, fe_selfdict) == 40, "funcexe_T fe_selfdict offset mismatch");
_Static_assert(offsetof(funcexe_T, fe_basetv) == 48, "funcexe_T fe_basetv offset mismatch");
_Static_assert(offsetof(funcexe_T, fe_found_var) == 56, "funcexe_T fe_found_var offset mismatch");

// --- typval_T layout assertions (Rust TypvalT must match exactly) ---
// typval_T size for provider.rs argvars array (3 * sizeof(typval_T)).
_Static_assert(sizeof(typval_T) == 16, "typval_T size mismatch: update provider.rs argvars stride");
_Static_assert(offsetof(typval_T, v_type) == 0, "typval_T v_type offset mismatch");
_Static_assert(offsetof(typval_T, v_lock) == 4, "typval_T v_lock offset mismatch");
_Static_assert(offsetof(typval_T, vval) == 8, "typval_T vval offset mismatch");

// --- evalarg_T layout assertions (Rust EvalargT must match exactly) ---
_Static_assert(sizeof(evalarg_T) == 32, "evalarg_T size mismatch: Rust EvalargT must be updated");
_Static_assert(offsetof(evalarg_T, eval_flags) == 0, "evalarg_T eval_flags offset mismatch");
_Static_assert(offsetof(evalarg_T, eval_getline) == 8, "evalarg_T eval_getline offset mismatch");
_Static_assert(offsetof(evalarg_T, eval_cookie) == 16, "evalarg_T eval_cookie offset mismatch");
_Static_assert(offsetof(evalarg_T, eval_tofree) == 24, "evalarg_T eval_tofree offset mismatch");

// --- partial_T field layout assertions (Rust PartialT must match exactly) ---
_Static_assert(offsetof(partial_T, pt_refcount) == 0, "partial_T pt_refcount offset mismatch");
_Static_assert(offsetof(partial_T, pt_copyID) == 4, "partial_T pt_copyID offset mismatch");
_Static_assert(offsetof(partial_T, pt_name) == 8, "partial_T pt_name offset mismatch");
_Static_assert(offsetof(partial_T, pt_func) == 16, "partial_T pt_func offset mismatch");
_Static_assert(offsetof(partial_T, pt_auto) == 24, "partial_T pt_auto offset mismatch");
_Static_assert(offsetof(partial_T, pt_argc) == 28, "partial_T pt_argc offset mismatch");
_Static_assert(offsetof(partial_T, pt_argv) == 32, "partial_T pt_argv offset mismatch");
_Static_assert(offsetof(partial_T, pt_dict) == 40, "partial_T pt_dict offset mismatch");
_Static_assert(sizeof(partial_T) == 48, "partial_T size mismatch");

// --- dict_T field layout assertions ---
_Static_assert(offsetof(dict_T, dv_lock) == 0, "dict_T dv_lock offset mismatch");
_Static_assert(offsetof(dict_T, dv_scope) == 4, "dict_T dv_scope offset mismatch");
_Static_assert(offsetof(dict_T, dv_refcount) == 8, "dict_T dv_refcount offset mismatch");
_Static_assert(offsetof(dict_T, dv_copyID) == 12, "dict_T dv_copyID offset mismatch");
_Static_assert(offsetof(dict_T, dv_hashtab) == 16, "dict_T dv_hashtab offset mismatch");

// --- list_T field layout assertions ---
_Static_assert(offsetof(list_T, lv_first) == 0, "list_T lv_first offset mismatch");
_Static_assert(offsetof(list_T, lv_copyID) == 68, "list_T lv_copyID offset mismatch");

// --- CallbackReader field layout assertions ---
_Static_assert(offsetof(CallbackReader, cb) == 0, "CallbackReader cb offset mismatch");
_Static_assert(offsetof(CallbackReader, self) == 16, "CallbackReader self offset mismatch");
_Static_assert(offsetof(CallbackReader, buffer) == 24, "CallbackReader buffer offset mismatch");
_Static_assert(offsetof(CallbackReader, buffered) == 49, "CallbackReader buffered offset mismatch");

// --- Callback layout assertions (validated by Rust CallbackT #[repr(C)]) ---
_Static_assert(sizeof(Callback) == 16, "Callback size must be 16 bytes");
_Static_assert(offsetof(Callback, data) == 0, "Callback.data must be at offset 0");
_Static_assert(offsetof(Callback, type) == 8, "Callback.type must be at offset 8");

// --- sctx_T layout assertions ---
_Static_assert(sizeof(sctx_T) == 24, "sctx_T size must be 24 bytes");

// --- NvimCursorVisualState layout assertions ---
_Static_assert(sizeof(NvimCursorVisualState) == 40,
               "NvimCursorVisualState size mismatch: expected 40 bytes");

// --- NvimTimerFields layout assertions ---
_Static_assert(offsetof(NvimTimerFields, timer_id) == 0, "NvimTimerFields.timer_id offset");
_Static_assert(offsetof(NvimTimerFields, repeat_count) == 4, "NvimTimerFields.repeat_count offset");
_Static_assert(offsetof(NvimTimerFields, refcount) == 8, "NvimTimerFields.refcount offset");
_Static_assert(offsetof(NvimTimerFields, emsg_count) == 12, "NvimTimerFields.emsg_count offset");
_Static_assert(offsetof(NvimTimerFields, timeout) == 16, "NvimTimerFields.timeout offset");
_Static_assert(offsetof(NvimTimerFields, stopped) == 24, "NvimTimerFields.stopped offset");
_Static_assert(offsetof(NvimTimerFields, paused) == 25, "NvimTimerFields.paused offset");
