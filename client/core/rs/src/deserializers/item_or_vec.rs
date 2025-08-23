//! # Item or Vec<Item> deserializer.
//!
//! Used to convert `item: T` (struct / map) -> `item: Vec<T>` (seq) in schemas with backward compatibility.
//! Supports deserializing either a T as Vec<T> with length 1, or a seq as Vec<T> directly.

use serde::{
  Deserialize, Deserializer,
  de::{
    DeserializeOwned, IntoDeserializer, Visitor,
    value::{MapAccessDeserializer, SeqAccessDeserializer},
  },
};

pub fn item_or_vec_deserializer<'de, D, T>(
  deserializer: D,
) -> Result<Vec<T>, D::Error>
where
  D: Deserializer<'de>,
  T: DeserializeOwned,
{
  deserializer
    .deserialize_any(ItemOrVecVisitor::<T>(std::marker::PhantomData))
}

pub fn option_item_or_vec_deserializer<'de, D, T>(
  deserializer: D,
) -> Result<Option<Vec<T>>, D::Error>
where
  D: Deserializer<'de>,
  T: DeserializeOwned,
{
  deserializer.deserialize_any(OptionItemOrVecVisitor::<T>(
    std::marker::PhantomData,
  ))
}

struct ItemOrVecVisitor<T>(std::marker::PhantomData<T>);

impl<'de, T> Visitor<'de> for ItemOrVecVisitor<T>
where
  T: Deserialize<'de>,
{
  type Value = Vec<T>;

  fn expecting(
    &self,
    formatter: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(formatter, "Item or Vec<Item>")
  }

  fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::MapAccess<'de>,
  {
    T::deserialize(
      MapAccessDeserializer::new(map).into_deserializer(),
    )
    .map(|r| vec![r])
  }

  fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::SeqAccess<'de>,
  {
    Vec::<T>::deserialize(
      SeqAccessDeserializer::new(seq).into_deserializer(),
    )
  }
}

struct OptionItemOrVecVisitor<T>(std::marker::PhantomData<T>);

impl<'de, T> Visitor<'de> for OptionItemOrVecVisitor<T>
where
  T: Deserialize<'de>,
{
  type Value = Option<Vec<T>>;

  fn expecting(
    &self,
    formatter: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(formatter, "null or Item or Vec<Item>")
  }

  fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::MapAccess<'de>,
  {
    ItemOrVecVisitor::<T>(std::marker::PhantomData)
      .visit_map(map)
      .map(Some)
  }

  fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::SeqAccess<'de>,
  {
    ItemOrVecVisitor::<T>(std::marker::PhantomData)
      .visit_seq(seq)
      .map(Some)
  }
}
