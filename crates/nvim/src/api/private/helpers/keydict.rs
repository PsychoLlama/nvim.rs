//! Keydicts: the generated structs the typed `nvim_*` signatures take their
//! options as.
//!
//! There is one such struct per function, so the code here works through
//! [`KeySet`] instead: each field is found by key and borrowed as the typed
//! [`Slot`] it is. Every field is an `Option`, and `None` is the key the
//! caller did not name.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::{api_object_to_bool, api_typename, object_to_hl_id};
use crate::api::private::keyset::{FieldKind, KeySet, Slot};
use crate::api::private::validate::err_expected;
use crate::api_error;
use crate::narrow::number_as_int;
use crate::types::{ApiDict, Error, Float, KeyValuePair, Object, kErrorTypeValidation};

/// Fill the keyset `retval` from `dict`, type-checking each value against
/// what the field it names holds. Refuses at the first unknown key or wrong
/// type.
///
/// The dictionary is **consumed**: a value that matches its field moves into
/// it, and whatever the walk did not reach is released with the rest.
pub(crate) fn api_dict_to_keydict(retval: &mut dyn KeySet, dict: ApiDict) -> Result<(), Error> {
    let fields = retval.fields();
    for KeyValuePair { key, value: given } in dict {
        let Some(index) = retval.find(key.bytes()) else {
            let name = key.as_c_str().to_string_lossy();
            return Err(api_error!(kErrorTypeValidation, "Invalid key: '{name}'"));
        };
        let field = &fields[index];
        // A mismatch reports the field's name, which is the key's.
        let got = api_typename(given.kind());
        let wrong_type = || {
            err_expected(
                field.name,
                api_typename(field.kind.object_type()),
                Some(got),
            )
        };

        // Writing `Some` is what records that the caller named the key: a
        // field nobody wrote is still the `None` `Default` left.
        match retval.slot(index) {
            // An any-typed field takes the object as it stands.
            Slot::Any(slot) => *slot = Some(given),
            Slot::Integer(slot) if field.kind == FieldKind::HlGroup => {
                let mut hl_id = 0;
                if !given.is_nil() {
                    hl_id = object_to_hl_id(&given, key.as_c_str())?;
                }
                *slot = Some(hl_id.into());
            }
            Slot::Integer(slot) => *slot = Some(given.as_integer().ok_or_else(wrong_type)?),
            // A float field takes an integer too.
            Slot::Float(slot) => {
                let widened = given.as_integer().map(|n| n as Float);
                *slot = Some(given.as_float().or(widened).ok_or_else(wrong_type)?);
            }
            Slot::Boolean(slot) => *slot = Some(api_object_to_bool(&given, field.name, false)?),
            Slot::String(slot) => *slot = Some(given.into_string().ok_or_else(wrong_type)?),
            Slot::Array(slot) => *slot = Some(given.into_array().ok_or_else(wrong_type)?),
            Slot::Dict(slot) => {
                // An empty array is how msgpack spells an empty map.
                let empty = given.as_array().is_some_and(|array| array.is_empty());
                let pairs = if empty {
                    Some(ApiDict::EMPTY)
                } else {
                    given.into_dict()
                };
                *slot = Some(pairs.ok_or_else(wrong_type)?);
            }
            Slot::Handle(slot) => {
                // A handle arrives either under its own variant or as a plain
                // integer, and both carry it as an `Integer`.
                let handle = match given.as_integer() {
                    Some(n) => n,
                    None if given.kind() == field.kind.object_type() => {
                        given.as_handle().unwrap_or(0)
                    }
                    None => return Err(wrong_type()),
                };
                *slot = Some(number_as_int(handle));
            }
            Slot::LuaRef(_) => {
                let name = key.as_c_str().to_string_lossy();
                return Err(api_error!(
                    kErrorTypeValidation,
                    "Invalid key: '{name}' is only allowed from Lua"
                ));
            }
            Slot::StringArray(_) => unreachable!("only ShaDa's own keysets hold a string array"),
        }
    }
    Ok(())
}

/// The reverse of [`api_dict_to_keydict`]: the keyset `value` as a plain
/// dictionary, holding only the fields that were set. Lua references are
/// skipped -- they mean nothing outside the Lua state -- but still named,
/// with a nil value.
///
/// The keyset keeps its fields; what the answer holds is copies.
pub(crate) fn api_keydict_to_dict(value: &mut dyn KeySet) -> ApiDict {
    let fields = value.fields();
    let mut rv = ApiDict::with_capacity(fields.len());
    for (index, field) in fields.iter().enumerate() {
        let val = match value.slot(index) {
            Slot::Any(Some(v)) => v.clone(),
            Slot::Integer(Some(n)) => Object::integer(*n),
            Slot::Float(Some(f)) => Object::float(*f),
            Slot::Boolean(Some(b)) => Object::boolean(*b),
            Slot::String(Some(s)) => Object::string(s.clone()),
            Slot::Array(Some(a)) => Object::array(a.clone()),
            Slot::Dict(Some(d)) => Object::dict(d.clone()),
            Slot::Handle(Some(h)) => match field.kind {
                FieldKind::Buffer => Object::buffer(*h),
                _ => Object::window(*h),
            },
            Slot::LuaRef(Some(_)) => Object::Nil,
            Slot::StringArray(Some(_)) => {
                unreachable!("only ShaDa's own keysets hold a string array")
            }
            // A field the caller never named is not a key of the answer.
            _ => continue,
        };
        rv.insert(field.name.to_bytes(), val);
    }
    rv
}
