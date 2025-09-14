use serde::{
  Deserialize, Deserializer,
  de::{IntoDeserializer, Visitor},
};

#[derive(Debug, Clone)]
pub struct ForgivingVec<T>(pub Vec<T>);

impl<T> ForgivingVec<T> {
  pub fn iter(&self) -> std::slice::Iter<'_, T> {
    self.0.iter()
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }
}

impl<T> Default for ForgivingVec<T> {
  fn default() -> Self {
    ForgivingVec(Vec::new())
  }
}

impl<T> IntoIterator for ForgivingVec<T> {
  type Item = T;
  type IntoIter = <Vec<T> as IntoIterator>::IntoIter;
  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

impl<T> FromIterator<T> for ForgivingVec<T> {
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    Self(Vec::from_iter(iter))
  }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for ForgivingVec<T> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_seq(ForgivingVecVisitor::<T>(
      std::marker::PhantomData,
    ))
  }
}

struct ForgivingVecVisitor<T>(std::marker::PhantomData<T>);

impl<'de, T: Deserialize<'de>> Visitor<'de>
  for ForgivingVecVisitor<T>
{
  type Value = ForgivingVec<T>;

  fn expecting(
    &self,
    formatter: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(formatter, "Vec<T>")
  }

  fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
  where
    S: serde::de::SeqAccess<'de>,
  {
    let mut res =
      Vec::with_capacity(seq.size_hint().unwrap_or_default());
    loop {
      match seq.next_element::<serde_json::Value>() {
        Ok(Some(value)) => {
          match T::deserialize(value.clone().into_deserializer()) {
            Ok(item) => res.push(item),
            Err(e) => {
              // Since this is used to parse startup config (including logging config),
              // the tracing logging is not initialized. Need to use eprintln.
              eprintln!(
                "WARN: failed to parse item in list | {value:?} | {e:?}",
              )
            }
          }
        }
        Ok(None) => break,
        Err(e) => {
          eprintln!("WARN: failed to get item in list | {e:?}");
        }
      }
    }
    Ok(ForgivingVec(res))
  }
}
