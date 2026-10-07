use chrono::NaiveDate;
use color_eyre::Result;
use serde::{Deserialize, Deserializer};

pub mod date_format {
    use super::*;
    const FORMAT: &str = "%Y-%m-%d";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

macro_rules! json_field {
    ($map:expr, $field:expr) => {{
        let field_name: &str = $field;
        match $map.get(field_name) {
            Some(val) => ::serde::Deserialize::deserialize(val)
                .map_err(|e| ::color_eyre::eyre::eyre!("{field_name} invalid: {e}")),
            None => Err(::color_eyre::eyre::eyre!("{field_name} missing")),
        }
    }};
    ($map:expr, $field:expr, default = $default:expr) => {{
        let field_name: &str = $field;
        match $map.get(field_name) {
            Some(val) => ::serde::Deserialize::deserialize(val)
                .map_err(|e| ::color_eyre::eyre::eyre!("{field_name} invalid: {e}")),
            None => Ok($default),
        }
    }};
}
pub(crate) use json_field;

use crate::core::log::Logger;

pub fn fuzzy_match(a: impl Into<LangString>, b: impl Into<LangString>) -> f64 {
    let (a, b) = (clean(a), clean(b));
    let ret = strsim::normalized_levenshtein(&a, &b);
    Logger::debug(format!("levenshtein(A={a}, B={b}) = {ret}"));
    ret
}

const MIN_SIM: f64 = 0.75;
pub fn is_fuzzy_match(a: impl Into<LangString>, b: impl Into<LangString>) -> bool {
    fuzzy_match(a, b) >= MIN_SIM
}

fn clean(str: impl Into<LangString>) -> String {
    let ls: LangString = str.into();
    let s = ls.str.to_lowercase(); // Lowercase
    let s = s.trim_prefix("the"); // Remove leading 'the'
    let s = s
        .chars()
        .filter(|ch| !ch.is_ascii_punctuation())
        .collect::<String>(); // Remove punctuation
    if ls.lang.as_ref().is_some_and(|lang| lang == "ja")
        && kakasi::is_japanese(&s) >= kakasi::IsJapanese::Maybe
    {
        kakasi::convert(&s).romaji // Romanization
    } else {
        deunicode::deunicode(&s) // Transliteration
    }
}

pub struct LangString {
    str: String,
    lang: Option<String>,
}

impl LangString {
    pub fn new(str: &str, lang: &str) -> Self {
        Self {
            str: str.to_string(),
            lang: Some(lang.to_string()),
        }
    }
}

impl<T> From<T> for LangString
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        LangString {
            str: value.into(),
            lang: None,
        }
    }
}

pub struct AbortOnDrop(pub tokio::task::JoinHandle<()>);
impl Drop for AbortOnDrop {
    fn drop(&mut self) { self.0.abort(); }
}
