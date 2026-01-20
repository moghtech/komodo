use utoipa::OpenApi;

use crate::api::write;

#[derive(OpenApi)]
#[openapi(
  paths(
    // swarm
    // server
    // stack
    // deployment
    // build
    // repo
    // procedure
    // action
    // builder
    // alerter
    // resource_sync
    // alert
    // permissions
    // docker
  ),
)]
pub struct KomodoWriteApi;
