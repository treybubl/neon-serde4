use crate::date::{JsDate as JsDateConverter, DT};
use crate::errors::Error as LibError;
use chrono::{DateTime, TimeZone};
use neon::prelude::*;
use neon::types::JsDate;
use serde::{ser, ser::Serialize};
use serde_with::SerializeAs;
use std::marker::PhantomData;

impl<Tz: TimeZone> SerializeAs<DateTime<Tz>> for JsDateConverter {
    fn serialize_as<S>(source: &DateTime<Tz>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        DT(source).serialize(serializer)
    }
}

impl<Tz: TimeZone> SerializeAs<Option<DateTime<Tz>>> for JsDateConverter {
    fn serialize_as<S>(source: &Option<DateTime<Tz>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        source.as_ref().map(DT).serialize(serializer)
    }
}

#[doc(hidden)]
pub struct DateTimeSerializer<'a, 'j, C: 'a>
where
    C: Context<'j>,
{
    cx: &'a mut C,
    phantom: PhantomData<Handle<'j, JsDate>>,
}

pub(crate) const CHRONO_DATE_TIME_INTERNAL_NAME: &str = "neon_serde::chrono::DT\x08";

impl<'a, Tz: TimeZone> ser::Serialize for DT<&'a DateTime<Tz>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer
            .serialize_newtype_struct(CHRONO_DATE_TIME_INTERNAL_NAME, &self.0.timestamp_millis())
    }
}

#[doc(hidden)]
impl<'a, 'j, C> DateTimeSerializer<'a, 'j, C>
where
    C: Context<'j>,
{
    #[inline]
    pub(crate) fn new(cx: &'a mut C) -> Self {
        DateTimeSerializer {
            cx,
            phantom: Default::default(),
        }
    }
}

impl<'a, 'j, C> ser::Serializer for DateTimeSerializer<'a, 'j, C>
where
    C: Context<'j>,
{
    type Ok = Handle<'j, JsValue>;
    type Error = LibError;

    type SerializeSeq = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_i64(self, millis: i64) -> Result<Self::Ok, Self::Error> {
        self.cx
            .date(millis as f64)
            .map_err(|err| LibError::Serialize {
                msg: format!("{err}"),
            })
            .map(|date| date.upcast::<JsValue>())
    }

    // Required methods
    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        unimplemented!()
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        unimplemented!()
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        unimplemented!()
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        unimplemented!()
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        unimplemented!()
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        unimplemented!()
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        unimplemented!()
    }
}
