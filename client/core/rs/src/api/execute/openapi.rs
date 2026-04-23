use utoipa::OpenApi;

use crate::api::execute;

#[derive(OpenApi)]
#[openapi(
  paths(
    // swarm
    execute::remove_swarm_nodes,
    execute::update_swarm_node,
    execute::remove_swarm_stacks,
    execute::remove_swarm_services,
    execute::create_swarm_config,
    execute::rotate_swarm_config,
    execute::remove_swarm_configs,
    execute::create_swarm_secret,
    execute::rotate_swarm_secret,
    execute::remove_swarm_secrets,
    // server
    execute::start_container,
    execute::restart_container,
    execute::pause_container,
    execute::unpause_container,
    execute::stop_container,
    execute::destroy_container,
    execute::start_all_containers,
    execute::restart_all_containers,
    execute::pause_all_containers,
    execute::unpause_all_containers,
    execute::stop_all_containers,
    execute::prune_containers,
    execute::delete_network,
    execute::prune_networks,
    execute::delete_image,
    execute::prune_images,
    execute::delete_volume,
    execute::prune_volumes,
    execute::prune_docker_builders,
    execute::prune_buildx,
    execute::prune_system,
    // stack
    execute::deploy_stack,
    execute::batch_deploy_stack,
    execute::deploy_stack_if_changed,
    execute::batch_deploy_stack_if_changed,
    execute::pull_stack,
    execute::batch_pull_stack,
    execute::start_stack,
    execute::restart_stack,
    execute::pause_stack,
    execute::unpause_stack,
    execute::stop_stack,
    execute::destroy_stack,
    execute::run_stack_service,
    execute::batch_destroy_stack,
    // deployment
    execute::deploy,
    execute::batch_deploy,
    execute::pull_deployment,
    execute::start_deployment,
    execute::restart_deployment,
    execute::pause_deployment,
    execute::unpause_deployment,
    execute::stop_deployment,
    execute::destroy_deployment,
    execute::batch_destroy_deployment,
    // build
    execute::run_build,
    execute::batch_run_build,
    execute::cancel_build,
    // repo
    execute::clone_repo,
    execute::batch_clone_repo,
    execute::pull_repo,
    execute::batch_pull_repo,
    execute::build_repo,
    execute::batch_build_repo,
    execute::cancel_repo_build,
    // procedure
    execute::run_procedure,
    execute::batch_run_procedure,
    // action
    execute::run_action,
    execute::batch_run_action,
    // alerter
    execute::test_alerter,
    execute::send_alert,
    // resource_sync
    execute::run_sync,
    // maintenance
    execute::clear_repo_cache,
    execute::backup_core_database,
    execute::global_auto_update,
    execute::rotate_all_server_keys,
    execute::rotate_core_keys,
  ),
)]
pub struct KomodoExecuteApi;
