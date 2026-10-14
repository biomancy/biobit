use crate::{Error, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

/// Non-empty printable ASCII text used in project fields.
///
/// Characters must be in `0x20..=0x7E`. Internal spaces are preserved, but
/// whitespace at the start or end is rejected.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(transparent)]
pub struct Text(String);

impl Text {
    /// Creates non-empty printable ASCII text without surrounding whitespace.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.is_empty() {
            return Err(Error::EmptyValue);
        }
        let bytes = value.as_bytes();
        if bytes.first().is_some_and(|byte| byte.is_ascii_whitespace())
            || bytes.last().is_some_and(|byte| byte.is_ascii_whitespace())
        {
            return Err(Error::invalid(
                "Text",
                "must not start or end with whitespace",
            ));
        }
        if bytes.iter().any(|&byte| !(b' '..=b'~').contains(&byte)) {
            return Err(Error::invalid(
                "Text",
                "must contain only printable ASCII characters",
            ));
        }
        Ok(Self(value))
    }

    /// Returns the text as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the text as a `String`.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Borrow<String> for Text {
    fn borrow(&self) -> &String {
        &self.0
    }
}

impl Borrow<str> for Text {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Deref for Text {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Display for Text {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<Text> for String {
    fn from(value: Text) -> Self {
        value.into_inner()
    }
}

impl FromStr for Text {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<String> for Text {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Text {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for Text {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::Text;
    use std::collections::BTreeMap;

    #[test]
    fn preserves_internal_spaces_and_punctuation() {
        let value = "Drug response (24 h) ~ control";
        let text = Text::new(value).unwrap();
        assert_eq!(text.as_str(), value);
        let encoded = serde_json::to_string(&text).unwrap();
        assert_eq!(serde_json::from_str::<Text>(&encoded).unwrap(), text);
    }

    #[test]
    fn rejects_empty_non_ascii_and_whitespace_at_the_edges() {
        for value in ["", " leading", "trailing ", " ", "A\tB", "37°C"] {
            assert!(Text::new(value).is_err(), "{value:?}");
        }
        assert!(serde_json::from_str::<Text>(r#"" leading""#).is_err());
    }

    #[test]
    fn supports_borrowed_string_keys() {
        let values = BTreeMap::from([(Text::new("condition").unwrap(), true)]);
        assert_eq!(values.get("condition"), Some(&true));
    }
}
