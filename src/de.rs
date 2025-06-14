//!
//! Deserialize a `JsValue` into a Rust data structure
//!

use crate::errors::{Error as LibError, Result as LibResult};
use neon::types::JsDate;
use neon::{prelude::*, types::buffer::TypedArray};
use serde::{
    de::{
        DeserializeOwned, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, Unexpected,
        VariantAccess, Visitor,
    },
    forward_to_deserialize_any,
};

/// Deserialize an instance of type `T` from a `Handle<JsValue>`
///
/// # Errors
///
/// Can fail for various reasons see `Error`
///
pub fn from_value<'j, C, T>(cx: &mut C, value: Handle<'j, JsValue>) -> LibResult<T>
where
    C: Context<'j>,
    T: DeserializeOwned + ?Sized,
{
    let mut deserializer: Deserializer<C> = Deserializer::new(cx, value);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

/// Deserialize an instance of type `T` from an `Option<Handle<JsValue>>`
///
/// # Errors
///
/// Can fail for various reasons see `Error`
///
pub fn from_value_opt<'j, C, T>(cx: &mut C, value: Option<Handle<'j, JsValue>>) -> LibResult<T>
where
    C: Context<'j>,
    T: DeserializeOwned + ?Sized,
{
    let unwrapped = value.unwrap_or_else(|| JsUndefined::new(cx).upcast());
    from_value(cx, unwrapped)
}

#[doc(hidden)]
pub struct Deserializer<'a, 'j, C: Context<'j> + 'a> {
    cx: &'a mut C,
    input: Handle<'j, JsValue>,
}

#[doc(hidden)]
impl<'a, 'j, C: Context<'j>> Deserializer<'a, 'j, C> {
    fn new(cx: &'a mut C, input: Handle<'j, JsValue>) -> Self {
        Deserializer { cx, input }
    }
}

#[doc(hidden)]
impl<'x, 'd, 'a, 'j, C: Context<'j>> serde::de::Deserializer<'x>
    for &'d mut Deserializer<'a, 'j, C>
{
    type Error = LibError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'x>,
    {
        if self.input.downcast::<JsNull, C>(self.cx).is_ok()
            || self.input.downcast::<JsUndefined, C>(self.cx).is_ok()
        {
            visitor.visit_unit()
        } else if let Ok(val) = self.input.downcast::<JsBoolean, C>(self.cx) {
            visitor.visit_bool(val.value(self.cx))
        } else if let Ok(val) = self.input.downcast::<JsString, C>(self.cx) {
            visitor.visit_string(val.value(self.cx))
        } else if let Ok(val) = self.input.downcast::<JsNumber, C>(self.cx) {
            let v = val.value(self.cx);
            #[allow(clippy::float_cmp)]
            if v.trunc() == v {
                #[allow(clippy::cast_possible_truncation)]
                visitor.visit_i64(v as i64)
            } else {
                visitor.visit_f64(v)
            }
        } else if let Ok(_val) = self.input.downcast::<JsBuffer, C>(self.cx) {
            self.deserialize_bytes(visitor)
        } else if let Ok(val) = self.input.downcast::<JsArray, C>(self.cx) {
            let mut deserializer = JsArrayAccess::new(self.cx, val);
            visitor.visit_seq(&mut deserializer)
        } else if let Ok(val) = self.input.downcast::<JsDate, C>(self.cx) {
            let v = val.value(self.cx);
            #[allow(clippy::float_cmp)]
            if v.trunc() == v {
                #[allow(clippy::cast_possible_truncation)]
                visitor.visit_i64(v as i64)
            } else {
                visitor.visit_f64(v)
            }
        } else if let Ok(val) = self.input.downcast::<JsObject, C>(self.cx) {
            let mut deserializer = JsObjectAccess::new(self.cx, val)?;
            visitor.visit_map(&mut deserializer)
        } else {
            Err(LibError::NotImplemented {
                name: "unimplemented Deserializer::Deserializer",
            })
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'x>,
    {
        if self.input.downcast::<JsNull, C>(self.cx).is_ok()
            || self.input.downcast::<JsUndefined, C>(self.cx).is_ok()
        {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'x>,
    {
        if let Ok(val) = self.input.downcast::<JsString, C>(self.cx) {
            let key_value = val.value(self.cx);
            visitor.visit_enum(JsEnumAccess::new(self.cx, key_value, None))
        } else if let Ok(val) = self.input.downcast::<JsObject, C>(self.cx) {
            let prop_names = val.get_own_property_names(self.cx)?;
            let len = prop_names.len(self.cx);
            if len != 1 {
                return Err(LibError::InvalidKeyType {
                    key: format!("object key with {} properties", len),
                });
            }
            let key = prop_names
                .get::<JsValue, _, _>(self.cx, 0)?
                .downcast_or_throw::<JsString, C>(self.cx)?;
            let enum_value = val.get(self.cx, key)?;
            let key_value = key.value(self.cx);
            visitor.visit_enum(JsEnumAccess::new(self.cx, key_value, Some(enum_value)))
        } else {
            let m = self.input.to_string(self.cx)?.value(self.cx);
            Err(LibError::InvalidKeyType { key: m })
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'x>,
    {
        let buff = self.input.downcast_or_throw::<JsBuffer, C>(self.cx)?;
        let copy = Vec::from(buff.as_slice(self.cx));
        visitor.visit_bytes(&copy)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'x>,
    {
        let buff = self
            .input
            .downcast::<JsBuffer, C>(self.cx)
            .or_throw(self.cx)?;
        let copy = Vec::from(buff.as_slice(self.cx));
        visitor.visit_byte_buf(copy)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'x>,
    {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
       <V: Visitor<'x>>
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        unit unit_struct seq tuple tuple_struct map struct identifier
        newtype_struct
    }
}

#[doc(hidden)]
struct JsArrayAccess<'a, 'j, C: Context<'j> + 'a> {
    cx: &'a mut C,
    input: Handle<'j, JsArray>,
    idx: u32,
    len: u32,
}

#[doc(hidden)]
impl<'a, 'j, C: Context<'j>> JsArrayAccess<'a, 'j, C> {
    fn new(cx: &'a mut C, input: Handle<'j, JsArray>) -> Self {
        JsArrayAccess {
            len: input.len(cx),
            cx,
            input,
            idx: 0,
        }
    }
}

#[doc(hidden)]
impl<'x, 'a, 'j, C: Context<'j>> SeqAccess<'x> for JsArrayAccess<'a, 'j, C> {
    type Error = LibError;

    fn next_element_seed<T>(&mut self, seed: T) -> LibResult<Option<T::Value>>
    where
        T: DeserializeSeed<'x>,
    {
        if self.idx >= self.len {
            return Ok(None);
        }
        let v = self.input.get(self.cx, self.idx)?;
        self.idx += 1;

        let mut de = Deserializer::new(self.cx, v);
        seed.deserialize(&mut de).map(Some)
    }
}

#[doc(hidden)]
struct JsObjectAccess<'a, 'j, C: Context<'j> + 'a> {
    cx: &'a mut C,
    input: Handle<'j, JsObject>,
    prop_names: Handle<'j, JsArray>,
    idx: u32,
    len: u32,
}

#[doc(hidden)]
impl<'x, 'a, 'j, C: Context<'j>> JsObjectAccess<'a, 'j, C> {
    fn new(cx: &'a mut C, input: Handle<'j, JsObject>) -> LibResult<Self> {
        let prop_names = input.get_own_property_names(cx)?;
        let len = prop_names.len(cx);

        Ok(JsObjectAccess {
            cx,
            input,
            prop_names,
            idx: 0,
            len,
        })
    }
}

#[doc(hidden)]
impl<'x, 'a, 'j, C: Context<'j>> MapAccess<'x> for JsObjectAccess<'a, 'j, C> {
    type Error = LibError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'x>,
    {
        if self.idx >= self.len {
            return Ok(None);
        }

        let prop_name = self.prop_names.get(self.cx, self.idx)?;

        let mut de = Deserializer::new(self.cx, prop_name);
        seed.deserialize(&mut de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'x>,
    {
        if self.idx >= self.len {
            return Err(LibError::ArrayIndexOutOfBounds {
                length: self.len,
                index: self.idx,
            });
        }
        let prop_name = self.prop_names.get::<JsString, _, _>(self.cx, self.idx)?;
        let value = self.input.get(self.cx, prop_name)?;

        self.idx += 1;
        let mut de = Deserializer::new(self.cx, value);
        let res = seed.deserialize(&mut de)?;
        Ok(res)
    }
}

#[doc(hidden)]
struct JsEnumAccess<'a, 'j, C: Context<'j> + 'a> {
    cx: &'a mut C,
    variant: String,
    value: Option<Handle<'j, JsValue>>,
}

#[doc(hidden)]
impl<'a, 'j, C: Context<'j>> JsEnumAccess<'a, 'j, C> {
    fn new(cx: &'a mut C, key: String, value: Option<Handle<'j, JsValue>>) -> Self {
        JsEnumAccess {
            cx,
            variant: key,
            value,
        }
    }
}

#[doc(hidden)]
impl<'x, 'a, 'j, C: Context<'j>> EnumAccess<'x> for JsEnumAccess<'a, 'j, C> {
    type Error = LibError;
    type Variant = JsVariantAccess<'a, 'j, C>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'x>,
    {
        use serde::de::IntoDeserializer;
        let variant = self.variant.into_deserializer();
        let variant_access = JsVariantAccess::new(self.cx, self.value);
        seed.deserialize(variant).map(|v| (v, variant_access))
    }
}

#[doc(hidden)]
struct JsVariantAccess<'a, 'j, C: Context<'j> + 'a> {
    cx: &'a mut C,
    value: Option<Handle<'j, JsValue>>,
}

#[doc(hidden)]
impl<'a, 'j, C: Context<'j>> JsVariantAccess<'a, 'j, C> {
    fn new(cx: &'a mut C, value: Option<Handle<'j, JsValue>>) -> Self {
        JsVariantAccess { cx, value }
    }
}

#[doc(hidden)]
impl<'x, 'a, 'j, C: Context<'j>> VariantAccess<'x> for JsVariantAccess<'a, 'j, C> {
    type Error = LibError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value {
            Some(val) => {
                let mut deserializer = Deserializer::new(self.cx, val);
                serde::de::Deserialize::deserialize(&mut deserializer)
            }
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'x>,
    {
        match self.value {
            Some(val) => {
                let mut deserializer = Deserializer::new(self.cx, val);
                seed.deserialize(&mut deserializer)
            }
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'x>,
    {
        match self.value {
            Some(handle) => {
                if let Ok(val) = handle.downcast::<JsArray, C>(self.cx) {
                    let mut deserializer = JsArrayAccess::new(self.cx, val);
                    visitor.visit_seq(&mut deserializer)
                } else {
                    Err(serde::de::Error::invalid_type(
                        Unexpected::Other("JsValue"),
                        &"tuple variant",
                    ))
                }
            }
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'x>,
    {
        match self.value {
            Some(handle) => {
                if let Ok(val) = handle.downcast::<JsObject, C>(self.cx) {
                    let mut deserializer = JsObjectAccess::new(self.cx, val)?;
                    visitor.visit_map(&mut deserializer)
                } else {
                    Err(serde::de::Error::invalid_type(
                        Unexpected::Other("JsValue"),
                        &"struct variant",
                    ))
                }
            }
            _ => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}
