use crate::date::{JsDate as JsDateConverter, DT};
use chrono::{DateTime, Utc};
use serde::de;
use serde_with::DeserializeAs;
use std::fmt;

impl<'de> DeserializeAs<'de, DateTime<Utc>> for JsDateConverter {
    fn deserialize_as<D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        <DT<DateTime<Utc>> as de::Deserialize>::deserialize(deserializer).map(|dt| dt.0)
    }
}

impl<'de> DeserializeAs<'de, Option<DateTime<Utc>>> for JsDateConverter {
    fn deserialize_as<D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        <Option<DT<DateTime<Utc>>> as de::Deserialize>::deserialize(deserializer)
            .map(|opt| opt.map(|dt| dt.0))
    }
}

struct DTVisitor;

impl<'de> de::Deserialize<'de> for DT<DateTime<Utc>> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        // Instantiate our Visitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of MyMap.
        deserializer.deserialize_i64(DTVisitor)
    }
}

impl<'de> de::Visitor<'de> for DTVisitor {
    // The type that our Visitor is going to produce.
    type Value = DT<DateTime<Utc>>;

    // Format a message stating what data this Visitor expects to receive.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a JS Date object")
    }

    fn visit_i64<E>(self, millis: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let date_time = DateTime::from_timestamp_millis(millis)
            .ok_or_else(|| E::custom("invalid date time"))?;
        Ok(DT(date_time))
    }

    fn visit_f64<E>(self, millis: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let date_time = DateTime::from_timestamp_millis(millis.round() as i64)
            .ok_or_else(|| E::custom("invalid date time"))?;
        Ok(DT(date_time))
    }
}
