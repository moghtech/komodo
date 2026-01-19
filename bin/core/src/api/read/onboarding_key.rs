use std::cmp::Ordering;

use anyhow::{Context, anyhow};
use database::mungos::find::find_collect;
use komodo_client::api::read::{
  ListOnboardingKeys, ListOnboardingKeysResponse,
};
use mogh_resolver::Resolve;
use reqwest::StatusCode;
use mogh_error::AddStatusCodeError;

use crate::{api::read::ReadArgs, state::db_client};

//

impl Resolve<ReadArgs> for ListOnboardingKeys {
  async fn resolve(
    self,
    ReadArgs { user: admin }: &ReadArgs,
  ) -> mogh_error::Result<ListOnboardingKeysResponse> {
    if !admin.admin {
      return Err(
        anyhow!("This call is admin only")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    let mut keys =
      find_collect(&db_client().onboarding_keys, None, None)
        .await
        .context(
          "Failed to query database for Server onboarding keys",
        )?;

    // No expiry keys first, followed
    keys.sort_by(|a, b| {
      if a.expires == b.expires {
        Ordering::Equal
      } else if a.expires == 0 {
        Ordering::Less
      } else if b.expires == 0 {
        Ordering::Greater
      } else {
        // Descending
        b.expires.cmp(&a.expires)
      }
    });

    Ok(keys)
  }
}
