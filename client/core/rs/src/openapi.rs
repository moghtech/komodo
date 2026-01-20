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
    // ======
    //  READ
    // ======
    read::get_version,
    read::get_core_info,
    read::list_git_providers_from_config,
    read::list_docker_registries_from_config,
    read::list_secrets,
    // swarm
    read::list_swarms,
    read::list_full_swarms,
    read::get_swarm,
    read::get_swarm_action_state,
    read::get_swarms_summary,
    // server
    read::list_servers,
    read::list_full_servers,
    read::get_server,
    read::get_server_action_state,
    read::get_servers_summary,
    // stack
    read::list_stacks,
    read::list_full_stacks,
    read::get_stack,
    read::get_stack_action_state,
    read::get_stacks_summary,
    // deployment
    read::list_deployments,
    read::list_full_deployments,
    read::get_deployment,
    read::get_deployment_action_state,
    read::get_deployments_summary,
    // build
    read::list_builds,
    read::list_full_builds,
    read::get_build,
    read::get_build_action_state,
    read::get_builds_summary,
    // repo
    read::list_repos,
    read::list_full_repos,
    read::get_repo,
    read::get_repo_action_state,
    read::get_repos_summary,
    // procedure
    read::list_procedures,
    read::list_full_procedures,
    read::get_procedure,
    read::get_procedure_action_state,
    read::get_procedures_summary,
    // action
    read::list_actions,
    read::list_full_actions,
    read::get_action,
    read::get_action_action_state,
    read::get_actions_summary,
    // builder
    read::list_builders,
    read::list_full_builders,
    read::get_builder,
    read::get_builders_summary,
    // alerter
    read::list_alerters,
    read::list_full_alerters,
    read::get_alerter,
    read::get_alerters_summary,
    // resource_sync
    read::list_resource_syncs,
    read::list_full_resource_syncs,
    read::get_resource_sync,
    read::get_resource_sync_action_state,
    read::get_resource_syncs_summary,
  ),
)]
pub struct KomodoApi;
