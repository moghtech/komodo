use mogh_auth_client::openapi::MoghAuthApi;
use utoipa::OpenApi;

use crate::api::{
  execute::openapi::KomodoExecuteApi, read::openapi::KomodoReadApi,
  user::KomodoUserApi, write::openapi::KomodoWriteApi,
};

#[derive(OpenApi)]
#[openapi(
  nest(
    (path = "/auth", api = MoghAuthApi),
    (path = "/user", api = KomodoUserApi),
    (path = "/read", api = KomodoReadApi),
    (path = "/execute", api = KomodoExecuteApi),
    (path = "/write", api = KomodoWriteApi),
  ),
)]
pub struct KomodoApi;
