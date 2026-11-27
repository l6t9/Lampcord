pub mod marker;

use serde::{
    Deserialize,
    de::{Deserializer, Error as DeError, Visitor},
    ser::{Serialize, Serializer},
};
use std::any;
use std::num;
use std::{
    cmp::Ordering,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    marker::PhantomData,
    num::ParseIntError,
    str::FromStr,
};

#[repr(transparent)]
pub struct Id<T> {
    value: u64,
    phantom: PhantomData<fn(T) -> T>,
}

impl<T> Id<T> {
    const fn from_u64(value: u64) -> Self {
        Self {
            value,
            phantom: PhantomData,
        }
    }

    pub const fn new(n: u64) -> Self {
        Self::from_u64(n)
    }

    pub const fn get(self) -> u64 {
        self.value
    }

    pub const fn cast<New>(self) -> Id<New> {
        Id::from_u64(self.value)
    }

    pub const fn created_at_millis(self) -> i64 {
        ((self.value >> 22) as i64) + DISCORD_EPOCH_MS
    }
}

pub const DISCORD_EPOCH_MS: i64 = 1_420_070_400_000;

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> Debug for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("Id<")?;
        let type_name = any::type_name::<T>();
        let name = type_name.rsplit("::").next().unwrap_or(type_name);
        f.write_str(name)?;
        f.write_str(">(")?;
        Display::fmt(&self.value, f)?;
        f.write_str(")")
    }
}

impl<T> Display for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.value, f)
    }
}

impl<T> Eq for Id<T> {}

impl<T> Hash for Id<T> {
    fn hash<U: Hasher>(&self, state: &mut U) {
        state.write_u64(self.value);
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> PartialEq<u64> for Id<T> {
    fn eq(&self, other: &u64) -> bool {
        self.value == *other
    }
}

impl<T> PartialEq<Id<T>> for u64 {
    fn eq(&self, other: &Id<T>) -> bool {
        *self == other.value
    }
}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> From<u64> for Id<T> {
    fn from(value: u64) -> Self {
        Self::from_u64(value)
    }
}

impl<T> From<Id<T>> for u64 {
    fn from(id: Id<T>) -> Self {
        id.value
    }
}

impl<T> FromStr for Id<T> {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u64>().map(Self::from_u64)
    }
}

impl<T> TryFrom<i64> for Id<T> {
    type Error = num::TryFromIntError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        u64::try_from(value).map(Self::from_u64)
    }
}

impl<T> Serialize for Id<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.value)
    }
}

struct IdVisitor<T> {
    phantom: PhantomData<fn(T) -> T>,
}

impl<T> IdVisitor<T> {
    const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<'de, T> Visitor<'de> for IdVisitor<T> {
    type Value = Id<T>;

    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("a snowflake ID as a string or integer")
    }

    fn visit_u64<E: DeError>(self, value: u64) -> Result<Self::Value, E> {
        Ok(Id::from_u64(value))
    }

    fn visit_i64<E: DeError>(self, value: i64) -> Result<Self::Value, E> {
        u64::try_from(value)
            .map(Id::from_u64)
            .map_err(|_| DeError::custom("ID must not be negative"))
    }

    fn visit_str<E: DeError>(self, value: &str) -> Result<Self::Value, E> {
        value
            .parse::<u64>()
            .map(Id::from_u64)
            .map_err(|_| DeError::custom("ID string is not a valid unsigned integer"))
    }
}

impl<'de, T> Deserialize<'de> for Id<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(IdVisitor::new())
    }
}

#[cfg(test)]
mod tests {
    use super::{Id, marker::UserMarker};

    #[test]
    fn serialize_is_string() {
        let id = Id::<UserMarker>::new(114941315417899012);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!("\"114941315417899012\"", json);
    }

    #[test]
    fn deserialize_from_string_and_int() {
        let from_str: Id<UserMarker> = serde_json::from_str("\"42\"").unwrap();
        let from_int: Id<UserMarker> = serde_json::from_str("42").unwrap();
        assert_eq!(42, from_str.get());
        assert_eq!(from_str, from_int);
    }

    #[test]
    fn debug_shows_marker() {
        let id = Id::<UserMarker>::new(1);
        assert_eq!("Id<UserMarker>(1)", format!("{id:?}"));
    }

    #[test]
    fn cast_preserves_value() {
        let id = Id::<UserMarker>::new(5);
        assert_eq!(5, id.cast::<super::marker::GuildMarker>().get());
    }
}
