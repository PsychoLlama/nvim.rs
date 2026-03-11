#pragma once

#include <stddef.h>  // IWYU pragma: keep

#include "nvim/api/private/defs.h"  // IWYU pragma: keep
#include "nvim/cmdexpand_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/garray_defs.h"
#include "nvim/option_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/runtime_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

/// Stack of execution contexts.  Each entry is an estack_T.
/// Current context is at ga_len - 1.
extern garray_T exestack;
#define HAVE_SOURCING_INFO  (exestack.ga_data != NULL && exestack.ga_len > 0)
/// name of error message source
#define SOURCING_NAME (((estack_T *)exestack.ga_data)[exestack.ga_len - 1].es_name)
/// line number in the message source or zero
#define SOURCING_LNUM (((estack_T *)exestack.ga_data)[exestack.ga_len - 1].es_lnum)

/// Growarray to store info about already sourced scripts.
extern garray_T script_items;
#define SCRIPT_ITEM(id) (((scriptitem_T **)script_items.ga_data)[(id) - 1])
#define SCRIPT_ID_VALID(id) ((id) > 0 && (id) <= script_items.ga_len)

/// last argument for do_source()
enum {
  DOSO_NONE = 0,
  DOSO_VIMRC = 1,  ///< loading vimrc file
};

/// Used for flags in do_in_path()
enum {
  DIP_ALL     = 0x01,   ///< all matches, not just the first one
  DIP_DIR     = 0x02,   ///< find directories instead of files
  DIP_ERR     = 0x04,   ///< give an error message when none found
  DIP_START   = 0x08,   ///< also use "start" directory in 'packpath'
  DIP_OPT     = 0x10,   ///< also use "opt" directory in 'packpath'
  DIP_NORTP   = 0x20,   ///< do not use 'runtimepath'
  DIP_NOAFTER = 0x40,   ///< skip "after" directories
  DIP_AFTER   = 0x80,   ///< only use "after" directories
  DIP_DIRFILE = 0x200,  ///< find both files and directories
};

/// Functions now implemented in Rust (src/nvim-rs/runtime/) but still called
/// from C. Declarations here because the auto-generated header only covers
/// functions defined in C.
#include <stdbool.h>
#include "nvim/types_defs.h"

void ex_scriptnames(exarg_T *eap);
char *autoload_name(const char *name, size_t name_len);
bool script_autoload(const char *name, size_t name_len, bool reload);
void free_scriptnames(void);
void free_autoload_scriptnames(void);
void scriptnames_slash_adjust(void);
int add_pack_dir_to_rtp(char *fname, bool is_pack);
int ExpandRTDir(char *pat, int flags, int *num_file, char ***file, char *dirnames[]);
int expand_runtime_cmd(char *pat, int *numMatches, char ***matches);
int ExpandPackAddDir(char *pat, int *num_file, char ***file);
void runtime_init(void);
int do_in_path_and_pp(char *path, char *name, int flags, DoInRuntimepathCB callback, void *cookie);
int do_in_runtimepath(char *name, int flags, DoInRuntimepathCB callback, void *cookie);
int source_runtime(char *name, int flags);
int source_runtime_vim_lua(char *name, int flags);
int source_in_path_vim_lua(char *path, char *name, int flags);
int gen_expand_wildcards_and_cb(int num_pat, char **pats, int flags, bool all, DoInRuntimepathCB callback, void *cookie);
void add_pack_start_dirs(void);
void load_start_packages(void);
void ex_packloadall(exarg_T *eap);
void load_plugins(void);
void ex_packadd(exarg_T *eap);
bool pack_has_entries(char *buf);
int load_pack_plugin(bool opt, char *fname);
void ex_runtime(exarg_T *eap);
void set_context_in_runtime_cmd(expand_T *xp, const char *arg);

#include "runtime.h.generated.h"
