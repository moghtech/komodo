use komodo_client::entities::NoData;
use noise::key::SpkiPublicKey;
use periphery_client::api::keys::{
  RotateCorePublicKey, RotatePrivateKey, RotatePrivateKeyResponse,
};
use resolver_api::Resolve;

use crate::{
  config::periphery_config,
  state::{core_public_keys, periphery_keys},
};

//

impl Resolve<crate::api::Args> for RotatePrivateKey {
  #[instrument(
    "RotatePrivateKey",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<RotatePrivateKeyResponse> {
    let public_key = periphery_keys().rotate().await?.into_inner();
    info!("New Public Key: {public_key}");
    Ok(RotatePrivateKeyResponse { public_key })
  }
}

//

impl Resolve<crate::api::Args> for RotateCorePublicKey {
  #[instrument(
    "RotateCorePublicKey",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      public_key = self.public_key,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<NoData> {
    let config = periphery_config();

    let Some(core_public_keys_spec) =
      config.core_public_keys.as_ref()
    else {
      return Ok(NoData {});
    };

    let Some(path) = core_public_keys_spec
      .iter()
      // Finds the first Core Public Key in spec with `file` prefix.
      .find_map(|public_keys| public_keys.strip_prefix("file:"))
    else {
      return Ok(NoData {});
    };

    let public_key = SpkiPublicKey::from(self.public_key);

    // Check equality at path before trying to rewrite.
    match SpkiPublicKey::from_file(path) {
      Ok(existing) if existing == public_key => {}
      _ => public_key.write_pem_async(path).await?,
    }

    core_public_keys().refresh();

    Ok(NoData {})
  }
}
