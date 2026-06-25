use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, de};

struct FlexibleDateTime(DateTime<Utc>);

impl<'de> Deserialize<'de> for FlexibleDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FlexibleDateTimeVisitor;

        impl<'de> de::Visitor<'de> for FlexibleDateTimeVisitor {
            type Value = FlexibleDateTime;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .write_str("an RFC3339 timestamp string or non-negative integer milliseconds")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value < 0 {
                    return Err(E::custom(format!(
                        "timestamp milliseconds must be non-negative: {value}"
                    )));
                }

                self.visit_u64(value as u64)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let millis = i64::try_from(value)
                    .map_err(|_| E::custom(format!("timestamp milliseconds too high: {value}")))?;
                DateTime::<Utc>::from_timestamp_millis(millis)
                    .map(FlexibleDateTime)
                    .ok_or_else(|| E::custom(format!("invalid timestamp milliseconds: {value}")))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                DateTime::parse_from_rfc3339(value)
                    .map(|datetime| FlexibleDateTime(datetime.with_timezone(&Utc)))
                    .map_err(E::custom)
            }
        }

        deserializer.deserialize_any(FlexibleDateTimeVisitor)
    }
}

pub(crate) mod datetime_rfc3339_or_millis {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer};

    use super::FlexibleDateTime;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        FlexibleDateTime::deserialize(deserializer).map(|datetime| datetime.0)
    }
}

pub(crate) mod datetime_option_rfc3339_or_millis {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer};

    use super::FlexibleDateTime;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<FlexibleDateTime>::deserialize(deserializer)
            .map(|datetime| datetime.map(|datetime| datetime.0))
    }
}

pub(crate) mod float_without_decimal {
    use serde::Serializer;

    pub fn serialize<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if value.fract() == 0.0 {
            serializer.serialize_i64(*value as i64)
        } else {
            serializer.serialize_f64(*value)
        }
    }
}

pub(crate) mod price_option {
    use serde::{Deserialize, de};

    use super::super::price::Price;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Price>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let opt_price_f64 = Option::<f64>::deserialize(deserializer)?;

        match opt_price_f64 {
            None => Ok(None),
            Some(price_f64) => {
                if price_f64 == 0.0 {
                    Ok(None)
                } else {
                    match Price::try_from(price_f64) {
                        Ok(price) => Ok(Some(price)),
                        Err(e) => Err(de::Error::custom(e.to_string())),
                    }
                }
            }
        }
    }
}

pub(crate) mod client_id_option {
    use serde::{Deserialize, Deserializer};

    use crate::shared::models::client_id::ClientId;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ClientId>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) if s.is_empty() => Ok(None),
            Some(s) => ClientId::try_from(s)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize)]
    struct RequiredTimestamp {
        #[serde(deserialize_with = "datetime_rfc3339_or_millis::deserialize")]
        time: DateTime<Utc>,
    }

    #[derive(Debug, Deserialize)]
    struct OptionalTimestamp {
        #[serde(
            default,
            deserialize_with = "datetime_option_rfc3339_or_millis::deserialize"
        )]
        time: Option<DateTime<Utc>>,
    }

    #[test]
    fn datetime_rfc3339_or_millis_deserializes_milliseconds() {
        let timestamp: RequiredTimestamp = serde_json::from_str(r#"{ "time": 1747035005657 }"#)
            .expect("must deserialize timestamp millis");

        assert_eq!(timestamp.time.timestamp_millis(), 1747035005657);
    }

    #[test]
    fn datetime_rfc3339_or_millis_deserializes_rfc3339() {
        let timestamp: RequiredTimestamp =
            serde_json::from_str(r#"{ "time": "2025-05-12T07:30:05.657Z" }"#)
                .expect("must deserialize rfc3339 timestamp");

        assert_eq!(timestamp.time.timestamp_millis(), 1747035005657);
    }

    #[test]
    fn datetime_option_rfc3339_or_millis_deserializes_null_and_millis() {
        let timestamp: OptionalTimestamp =
            serde_json::from_str(r#"{ "time": null }"#).expect("must deserialize null timestamp");
        assert_eq!(timestamp.time, None);

        let timestamp: OptionalTimestamp = serde_json::from_str(r#"{ "time": 1747035005657 }"#)
            .expect("must deserialize optional millis timestamp");
        assert_eq!(timestamp.time.unwrap().timestamp_millis(), 1747035005657);
    }

    #[test]
    fn datetime_rfc3339_or_millis_rejects_negative_milliseconds() {
        let result: Result<RequiredTimestamp, _> = serde_json::from_str(r#"{ "time": -1 }"#);

        assert!(result.is_err());
    }

    #[test]
    fn datetime_rfc3339_or_millis_rejects_float_milliseconds() {
        let result: Result<RequiredTimestamp, _> =
            serde_json::from_str(r#"{ "time": 1747035005657.0 }"#);

        assert!(result.is_err());
    }
}
