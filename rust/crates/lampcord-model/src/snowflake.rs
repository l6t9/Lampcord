use std::fmt;

pub type Snowflake = u64;

const DISCORD_EPOCH_MS: u64 = 1_420_070_400_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseSnowflakeError(String);

impl fmt::Display for ParseSnowflakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid Discord snowflake: {}", self.0)
    }
}

impl std::error::Error for ParseSnowflakeError {}

pub fn parse_snowflake(value: &str) -> Result<Snowflake, ParseSnowflakeError> {
    value
        .parse::<Snowflake>()
        .map_err(|_| ParseSnowflakeError(value.to_owned()))
}

pub fn snowflake_timestamp(id: Snowflake) -> u64 {
    (id >> 22) + DISCORD_EPOCH_MS
}

pub mod serde {
    use super::{Snowflake, parse_snowflake};
    use ::serde::de::{self, Visitor};
    use ::serde::{Deserializer, Serializer};
    use std::fmt;

    pub fn serialize<S>(value: &Snowflake, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Snowflake, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SnowflakeVisitor;

        impl<'de> Visitor<'de> for SnowflakeVisitor {
            type Value = Snowflake;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a Discord snowflake string or integer")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                parse_snowflake(value).map_err(de::Error::custom)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(value)
            }
        }

        deserializer.deserialize_any(SnowflakeVisitor)
    }

    pub mod option {
        use super::Snowflake;
        use ::serde::de::{self, Visitor};
        use ::serde::{Deserializer, Serializer};
        use std::fmt;

        pub fn serialize<S>(value: &Option<Snowflake>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match value {
                Some(id) => serializer.serialize_some(&id.to_string()),
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Snowflake>, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct OptionVisitor;

            impl<'de> Visitor<'de> for OptionVisitor {
                type Value = Option<Snowflake>;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("an optional Discord snowflake")
                }

                fn visit_none<E>(self) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(None)
                }

                fn visit_unit<E>(self) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(None)
                }

                fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    super::deserialize(deserializer).map(Some)
                }
            }

            deserializer.deserialize_option(OptionVisitor)
        }
    }

    pub mod vec {
        use super::{Snowflake, parse_snowflake};
        use ::serde::de::{self, SeqAccess, Visitor};
        use ::serde::{Deserialize, Deserializer, Serializer};
        use std::fmt;

        pub fn serialize<S>(values: &[Snowflake], serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.collect_seq(values.iter().map(ToString::to_string))
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Snowflake>, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct SnowflakeElement(Snowflake);

            impl<'de> Deserialize<'de> for SnowflakeElement {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    struct ElementVisitor;

                    impl<'de> Visitor<'de> for ElementVisitor {
                        type Value = SnowflakeElement;

                        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                            formatter.write_str("a Discord snowflake")
                        }

                        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            parse_snowflake(value)
                                .map(SnowflakeElement)
                                .map_err(de::Error::custom)
                        }

                        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            Ok(SnowflakeElement(value))
                        }
                    }

                    deserializer.deserialize_any(ElementVisitor)
                }
            }

            struct VecVisitor;

            impl<'de> Visitor<'de> for VecVisitor {
                type Value = Vec<Snowflake>;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a list of Discord snowflakes")
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let mut values = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                    while let Some(SnowflakeElement(id)) = seq.next_element()? {
                        values.push(id);
                    }
                    Ok(values)
                }
            }

            deserializer.deserialize_seq(VecVisitor)
        }
    }
}
