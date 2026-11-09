#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// The shared core: the editor's nouns no one module owns (positions,
// handles, values, the API's object model, buffers, windows, the Ex
// command record). A type with an owner lives in that module. Each name
// is re-exported below by name, so a clash is a compile error.
pub mod api;
pub mod api_error;
pub mod api_string;
pub mod buffer;
pub mod builders;
pub mod cmdidx;
pub mod eval;
pub mod ex_cmds;
pub mod key;
pub mod multiqueue_list;
pub mod pos;
pub mod time;
pub mod types;
pub mod typval;
pub mod vim;
pub mod window;

pub use self::api::{
    AdditionalDataBuilder, ApiDict, Array, Boolean, BufferHandle, DictBuilder,
    ERROR_TYPE_EXCEPTION, ERROR_TYPE_NONE, ERROR_TYPE_VALIDATION, ErrorType, HLGroupID, HlMessage,
    Integer, KeyValuePair, LuaRef, MessageType, OBJECT_TYPE_ARRAY, OBJECT_TYPE_BOOLEAN,
    OBJECT_TYPE_BUFFER, OBJECT_TYPE_DICT, OBJECT_TYPE_FLOAT, OBJECT_TYPE_INTEGER,
    OBJECT_TYPE_LUA_REF, OBJECT_TYPE_NIL, OBJECT_TYPE_STRING, OBJECT_TYPE_TABPAGE,
    OBJECT_TYPE_WINDOW, Object, ObjectType, TabpageHandle, WindowHandle, key_value_pair,
};
pub use self::api_error::Error;
pub use self::api_string::ApiString;
pub use self::buffer::{
    AlignTextPos, BfaFlags, BlnFlags, BorderTextType, BufName, BufUpdateCallbacks, Buffer,
    BufferRef, DiffBlock, DiffLineChange, DispTick, DoBufAction, DoBufStart, FLOAT_ANCHOR_EAST,
    FLOAT_ANCHOR_SOUTH, FLOAT_RELATIVE_CURSOR, FLOAT_RELATIVE_EDITOR, FLOAT_RELATIVE_LASTSTATUS,
    FLOAT_RELATIVE_MOUSE, FLOAT_RELATIVE_TABLINE, FLOAT_RELATIVE_WINDOW, FcsChars, FloatAnchor,
    FloatRelative, GetFileFlags, GetFileRet, LLPos, LcsChars, MatchItem, MatchState, PosSave,
    Shown, SynBlock, SynTime, Taggy, WinConfig, WinSplit, WinStyle, file_buffer_b_signcols,
};
pub(crate) use self::buffer::{DiffLine, DiffLineChanges, KeymapEntry};
pub use self::builders::{ArrayBuf, DictBuf};
pub use self::cmdidx::CmdIdx;
pub use self::eval::{
    ExprType, MessagePackType, NotAVimVar, SaveVEvent, Timer, VAR_FLAVOUR_DEFAULT,
    VAR_FLAVOUR_SESSION, VAR_FLAVOUR_SHADA, VarFlavour, VimVarFlags, Vv,
};
pub use self::ex_cmds::{
    CmdAddr, CmdFilter, CmdLine, CmdMod, CmdModFlags, CmdParseInfo, CmdParseMagic, EcmdCmd, ExArg,
    ExArgt,
};
pub use self::key::Key;
pub use self::pos::{ColNr, LPos, LineNr, Pos};
pub use self::time::Timestamp;
pub use self::types::{Float, Handle, NS, OptInt, ProfTime, ScreenAttr, ScreenChar};
pub use self::typval::{
    BOOL_VAR_FALSE, BOOL_VAR_TRUE, Blob, BoolVarValue, Callback, Dict, DictWatcher, FuncCall,
    LIST_LEN_MAY_KNOW, LIST_LEN_SHOULD_KNOW, LIST_LEN_UNKNOWN, List, ListItem, ListLenSpecials,
    ListWatch, Partial, Refcount, RefcountSize, SPECIAL_VAR_NULL, ScopeType, ScriptCtx, ScriptId,
    SpecialVarValue, TypVal, UVarNumber, UserFunc, VAR_BLOB, VAR_BOOL, VAR_DEF_SCOPE, VAR_DICT,
    VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NO_SCOPE, VAR_NUMBER, VAR_PARTIAL, VAR_SCOPE, VAR_SHORT_LEN,
    VAR_SPECIAL, VAR_STRING, VAR_TYPE_BLOB, VAR_TYPE_BOOL, VAR_TYPE_DICT, VAR_TYPE_FLOAT,
    VAR_TYPE_FUNC, VAR_TYPE_LIST, VAR_TYPE_NUMBER, VAR_TYPE_SPECIAL, VAR_TYPE_STRING, VAR_UNKNOWN,
    VarLock, VarNumber, VarType, VarTypeCode,
};
pub(crate) use self::typval::{FuncBody, FuncProfile};
pub use self::vim::{
    CD_SCOPE_GLOBAL, CD_SCOPE_INVALID, CD_SCOPE_TABPAGE, CD_SCOPE_WINDOW, CdCause, CdScope,
    Direction, Failed,
};
pub use self::window::{Frame, SwitchWin, Tabpage, WLine, WinExecute, WinInfo, WinOpt, Window};
