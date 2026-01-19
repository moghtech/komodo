use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{api::write::KomodoWriteRequest, entities::NoData};

//

/// **Admin only.** Close the Alert at the given id.
/// Response: [NoData]
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(NoData)]
#[error(mogh_error::Error)]
pub struct CloseAlert {
  /// The id of the Alert to close.
  pub id: String,
}
