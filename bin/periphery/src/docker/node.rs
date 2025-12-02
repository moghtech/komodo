use anyhow::Context;
use bollard::query_parameters::ListNodesOptions;
use komodo_client::entities::docker::node::*;

use super::{
  DockerClient, convert_platform, convert_resource_object,
};

impl DockerClient {
  /// Lists swarm nodes
  pub async fn list_swarm_nodes(
    &self,
  ) -> anyhow::Result<Vec<SwarmNodeListItem>> {
    let mut nodes = self
      .docker
      .list_nodes(Option::<ListNodesOptions>::None)
      .await
      .context("Failed to query for swarm node list")?
      .into_iter()
      .map(convert_node_list_item)
      .collect::<Vec<_>>();

    nodes.sort_by(|a, b| {
      a.state
        .cmp(&b.state)
        .then_with(|| a.name.cmp(&b.name))
        .then_with(|| a.hostname.cmp(&b.hostname))
    });

    Ok(nodes)
  }

  /// Inspect a swarm node
  pub async fn inspect_swarm_node(
    &self,
    node_name: &str,
  ) -> anyhow::Result<SwarmNode> {
    self
      .docker
      .inspect_node(node_name)
      .await
      .map(convert_node)
      .with_context(|| {
        format!(
          "Failed to query for swarm node with name {node_name}"
        )
      })
  }
}

fn convert_node_list_item(
  node: bollard::models::Node,
) -> SwarmNodeListItem {
  let (name, role, availability) = node
    .spec
    .map(|spec| {
      (
        spec.name,
        spec.role.map(convert_role),
        spec.availability.map(convert_availability),
      )
    })
    .unwrap_or_default();
  SwarmNodeListItem {
    id: node.id,
    name,
    role,
    availability,
    hostname: node
      .description
      .and_then(|description| description.hostname),
    state: node
      .status
      .and_then(|status| status.state.map(convert_state)),
    created_at: node.created_at,
    updated_at: node.updated_at,
  }
}

fn convert_node(node: bollard::models::Node) -> SwarmNode {
  SwarmNode {
    id: node.id,
    version: node.version.map(super::convert_object_version),
    created_at: node.created_at,
    updated_at: node.updated_at,
    spec: node.spec.map(convert_node_spec),
    description: node.description.map(|description| {
      NodeDescription {
        hostname: description.hostname,
        platform: description.platform.map(convert_platform),
        resources: description.resources.map(convert_resource_object),
        engine: description.engine.map(|engine| EngineDescription {
          engine_version: engine.engine_version,
          labels: engine.labels,
          plugins: engine.plugins.map(|plugins| {
            plugins
              .into_iter()
              .map(|plugin| EngineDescriptionPlugins {
                typ: plugin.typ,
                name: plugin.name,
              })
              .collect()
          }),
        }),
        tls_info: description.tls_info.map(super::convert_tls_info),
      }
    }),
    status: node.status.map(|status| NodeStatus {
      state: status.state.map(convert_state),
      message: status.message,
      addr: status.addr,
    }),
    manager_status: node.manager_status.map(|manager_status| {
      ManagerStatus {
        leader: manager_status.leader,
        reachability: manager_status.reachability.map(
          |reachability| match reachability {
            bollard::secret::Reachability::UNKNOWN => {
              NodeReachability::UNKNOWN
            }
            bollard::secret::Reachability::UNREACHABLE => {
              NodeReachability::UNREACHABLE
            }
            bollard::secret::Reachability::REACHABLE => {
              NodeReachability::REACHABLE
            }
          },
        ),
        addr: manager_status.addr,
      }
    }),
  }
}

fn convert_node_spec(spec: bollard::models::NodeSpec) -> NodeSpec {
  NodeSpec {
    name: spec.name,
    labels: spec.labels,
    role: spec.role.map(convert_role),
    availability: spec.availability.map(convert_availability),
  }
}

fn convert_role(
  role: bollard::secret::NodeSpecRoleEnum,
) -> NodeSpecRoleEnum {
  match role {
    bollard::secret::NodeSpecRoleEnum::EMPTY => {
      NodeSpecRoleEnum::EMPTY
    }
    bollard::secret::NodeSpecRoleEnum::WORKER => {
      NodeSpecRoleEnum::WORKER
    }
    bollard::secret::NodeSpecRoleEnum::MANAGER => {
      NodeSpecRoleEnum::MANAGER
    }
  }
}

fn convert_availability(
  availability: bollard::secret::NodeSpecAvailabilityEnum,
) -> NodeSpecAvailabilityEnum {
  match availability {
    bollard::secret::NodeSpecAvailabilityEnum::EMPTY => {
      NodeSpecAvailabilityEnum::EMPTY
    }
    bollard::secret::NodeSpecAvailabilityEnum::ACTIVE => {
      NodeSpecAvailabilityEnum::ACTIVE
    }
    bollard::secret::NodeSpecAvailabilityEnum::PAUSE => {
      NodeSpecAvailabilityEnum::PAUSE
    }
    bollard::secret::NodeSpecAvailabilityEnum::DRAIN => {
      NodeSpecAvailabilityEnum::DRAIN
    }
  }
}

fn convert_state(state: bollard::secret::NodeState) -> NodeState {
  match state {
    bollard::secret::NodeState::UNKNOWN => NodeState::UNKNOWN,
    bollard::secret::NodeState::DOWN => NodeState::DOWN,
    bollard::secret::NodeState::READY => NodeState::READY,
    bollard::secret::NodeState::DISCONNECTED => {
      NodeState::DISCONNECTED
    }
  }
}
