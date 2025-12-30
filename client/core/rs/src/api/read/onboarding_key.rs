use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::onboarding_key::OnboardingKey;

use super::KomodoReadRequest;

/// **Admin only.** Gets list of onboarding keys.
/// Response: [ListOnboardingKeysResponse]
#[typeshare]
#[derive(
  Debug, Clone, Serialize, Deserialize, Resolve, EmptyTraits,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListOnboardingKeysResponse)]
#[error(serror::Error)]
pub struct ListOnboardingKeys {}

#[typeshare]
pub type ListOnboardingKeysResponse = Vec<OnboardingKey>;
