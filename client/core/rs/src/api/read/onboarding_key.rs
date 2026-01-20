use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::onboarding_key::OnboardingKey;

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ListOnboardingKeys",
  description = "**Admin only.** Gets list of onboarding keys.",
  request_body(content = ListOnboardingKeys),
  responses(
    (status = 200, description = "The list of onboarding keys", body = ListOnboardingKeysResponse),
  ),
)]
pub fn list_onboarding_keys() {}

/// **Admin only.** Gets list of onboarding keys.
/// Response: [ListOnboardingKeysResponse]
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListOnboardingKeysResponse)]
#[error(mogh_error::Error)]
pub struct ListOnboardingKeys {}

#[typeshare]
pub type ListOnboardingKeysResponse = Vec<OnboardingKey>;
