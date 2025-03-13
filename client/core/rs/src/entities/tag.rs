use derive_builder::Builder;
use partial_derive2::Partial;
use serde::{Deserialize, Serialize};
use strum::AsRefStr;
use typeshare::typeshare;

use crate::entities::MongoId;

#[typeshare(serialized_as = "Partial<Tag>")]
pub type _PartialTag = PartialTag;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Builder, Partial)]
#[partial_derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(
  feature = "mongo",
  derive(mongo_indexed::derive::MongoIndexed)
)]
pub struct Tag {
  /// The Mongo ID of the tag.
  /// This field is de/serialized from/to JSON as
  /// `{ "_id": { "$oid": "..." }, ...(rest of serialized Tag) }`
  #[serde(
    default,
    rename = "_id",
    skip_serializing_if = "String::is_empty",
    with = "bson::serde_helpers::hex_string_as_object_id"
  )]
  #[builder(setter(skip))]
  pub id: MongoId,

  #[cfg_attr(feature = "mongo", unique_index)]
  pub name: String,

  /// Hex color code with alpha for UI display
  #[serde(default)]
  pub color: TagColor,

  #[serde(default)]
  #[builder(default)]
  #[cfg_attr(feature = "mongo", index)]
  pub owner: String,
}

#[typeshare]
#[derive(Serialize, Deserialize, Default, Debug, Clone, AsRefStr)]
pub enum TagColor {
  LightSlate,
  #[default]
  Slate,
  DarkSlate,

  LightRed,
  Red,
  DarkRed,

  LightOrange,
  Orange,
  DarkOrange,

  LightAmber,
  Amber,
  DarkAmber,

  LightYellow,
  Yellow,
  DarkYellow,

  LightLime,
  Lime,
  DarkLime,

  LightGreen,
  Green,
  DarkGreen,

  LightEmerald,
  Emerald,
  DarkEmerald,

  LightTeal,
  Teal,
  DarkTeal,

  LightCyan,
  Cyan,
  DarkCyan,

  LightSky,
  Sky,
  DarkSky,

  LightBlue,
  Blue,
  DarkBlue,

  LightIndigo,
  Indigo,
  DarkIndigo,

  LightViolet,
  Violet,
  DarkViolet,

  LightPurple,
  Purple,
  DarkPurple,

  LightFuchsia,
  Fuchsia,
  DarkFuchsia,

  LightPink,
  Pink,
  DarkPink,

  LightRose,
  Rose,
  DarkRose,
}
