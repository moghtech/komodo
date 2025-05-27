//! This is a module to deserialize [PermissionLevelAndSpecifics].
//!
//! ## As just [PermissionLevel]
//! permission = "Write"
//!
//! ## As expanded with [SpecificPermission]
//! permission = { level = "Write", specific = ["Terminal"] }

use std::str::FromStr;

use indexmap::IndexSet;
use serde::{
  Deserialize, Serialize,
  de::{Visitor, value::MapAccessDeserializer},
};

use crate::entities::permission::{
  PermissionLevel, PermissionLevelAndSpecifics, SpecificPermission,
};

#[derive(Serialize, Deserialize)]
struct _PermissionLevelAndSpecifics {
  #[serde(default)]
  level: PermissionLevel,
  #[serde(default)]
  specific: IndexSet<SpecificPermission>,
}

impl Serialize for PermissionLevelAndSpecifics {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    if self.specific.is_empty() {
      // Serialize to simple string
      self.level.serialize(serializer)
    } else {
      _PermissionLevelAndSpecifics {
        level: self.level,
        specific: self.specific.clone(),
      }
      .serialize(serializer)
    }
  }
}

impl<'de> Deserialize<'de> for PermissionLevelAndSpecifics {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_any(PermissionLevelAndSpecificsVisitor)
  }
}

struct PermissionLevelAndSpecificsVisitor;

impl<'de> Visitor<'de> for PermissionLevelAndSpecificsVisitor {
  type Value = PermissionLevelAndSpecifics;

  fn expecting(
    &self,
    formatter: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(
      formatter,
      "PermissionLevel or PermissionLevelAndSpecifics"
    )
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(PermissionLevelAndSpecifics {
      level: PermissionLevel::from_str(v)
        .map_err(|e| serde::de::Error::custom(e))?,
      specific: IndexSet::new(),
    })
  }

  fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::MapAccess<'de>,
  {
    _PermissionLevelAndSpecifics::deserialize(
      MapAccessDeserializer::new(map),
    )
    .map(|p| PermissionLevelAndSpecifics {
      level: p.level,
      specific: p.specific,
    })
  }

  fn visit_unit<E>(self) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(PermissionLevelAndSpecifics {
      level: PermissionLevel::None,
      specific: IndexSet::new(),
    })
  }

  fn visit_none<E>(self) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    self.visit_unit()
  }
}
