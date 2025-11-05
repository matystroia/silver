use chrono::NaiveDate;
use color_eyre::{Result, eyre::eyre};
use serde::{Deserialize, Deserializer, Serializer, de::DeserializeOwned};
use serde_json::{Map, Value};

pub mod date_format {
    use super::*;
    const FORMAT: &str = "%Y-%m-%d";

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.format(FORMAT).to_string();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

pub fn field<T: DeserializeOwned>(value: &Map<String, Value>, field_name: &str) -> Result<T> {
    let val = value
        .get(field_name)
        .ok_or_else(|| eyre!("{field_name} missing"))?;
    serde_json::from_value(val.clone()).map_err(|_| eyre!("{field_name} invalid"))
}

pub fn optional_field<T: DeserializeOwned>(
    value: &Map<String, Value>,
    field_name: &str,
) -> Result<Option<T>> {
    match value.get(field_name) {
        None => Ok(None),
        Some(val) => serde_json::from_value(val.clone()).map_err(|_| eyre!("{field_name} invalid")),
    }
}
