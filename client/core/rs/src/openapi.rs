use mogh_auth_client::openapi::MoghAuthApi;
use utoipa::OpenApi;

mod user {
  pub use crate::api::user::*;
}

mod read {
  pub use crate::api::read::*;
}

#[derive(OpenApi)]
#[openapi(
  nest(
    (path = "/auth", api = MoghAuthApi)
  ),
  paths(
    // ======
    //  USER
    // ======
    user::push_recently_viewed,
    user::set_last_seen_update,
    user::create_api_key,
    user::delete_api_key,
    // // ======
    // //  READ
    // // ======
    read::get_version,
    read::get_core_info,
    read::list_git_providers_from_config,
    read::list_docker_registries_from_config,
    read::list_secrets,
  ),
)]
pub struct KomodoApi;
