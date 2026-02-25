import { deploymentStateIntention, hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import DeploymentTable from "./table";
import NewResource from "@/resources/new";
import DeploymentTabs from "./tabs";
import {
  DeployDeployment,
  DestroyDeployment,
  PauseUnpauseDeployment,
  PullDeployment,
  RestartDeployment,
  StartStopDeployment,
} from "./executions";
import { useSwarm } from "@/resources/swarm";
import { useServer } from "@/resources/server";
import ResourceLink from "@/resources/link";
import { Divider, Group, Text } from "@mantine/core";
import { RunBuild } from "@/resources/build/executions";
import DockerResourceLink from "@/components/docker/link";
import SwarmResourceLink from "@/components/swarm/link";
import ContainerPorts from "@/components/docker/container-ports";
import DeploymentUpdateAvailable from "./update-available";
import ResourceHeader from "../header";
import BatchExecutions from "@/components/batch-executions";
import { useState } from "react";
import ResourceSelector from "../selector";
import NewResourceWithDeployTarget from "../new-with-deploy-target";

export function useDeployment(id: string | undefined) {
  return useRead("ListDeployments", {}).data?.find((r) => r.id === id);
}

export function useFullDeployment(id: string) {
  return useRead("GetDeployment", { deployment: id }).data;
}

export const DeploymentComponents: RequiredResourceComponents<
  Types.DeploymentConfig,
  Types.DeploymentInfo,
  Types.DeploymentListItemInfo
> = {
  useList: () => useRead("ListDeployments", {}).data,
  useListItem: useDeployment,
  useFull: useFullDeployment,

  useResourceLinks: (deployment) => deployment?.config?.links,

  useDashboardSummaryData: () => {
    const summary = useRead("GetDeploymentsSummary", {}).data;
    const all = [
      summary?.running ?? 0,
      summary?.stopped ?? 0,
      summary?.unhealthy ?? 0,
      summary?.unknown ?? 0,
    ];
    const [running, stopped, unhealthy, unknown] = all;
    return [
      all.every((item) => item === 0) && {
        title: "Not Deployed",
        intention: "Neutral",
        value: summary?.not_deployed ?? 0,
      },
      { intention: "Good", value: running, title: "Running" },
      {
        title: "Stopped",
        intention: "Warning",
        value: stopped,
      },
      {
        title: "Unhealthy",
        intention: "Critical",
        value: unhealthy,
      },
      {
        title: "Unknown",
        intention: "Unknown",
        value: unknown,
      },
    ];
  },

  Description: () => (
    <>Connect deployments for alerting, building, and deploying.</>
  ),

  New: (props) => <NewResourceWithDeployTarget type="Deployment" {...props} />,

  BatchExecutions: () => (
    <BatchExecutions
      type="Deployment"
      executions={[
        "CheckDeploymentForUpdate",
        "PullDeployment",
        "Deploy",
        "RestartDeployment",
        "StopDeployment",
        "DestroyDeployment",
      ]}
    />
  ),

  Table: DeploymentTable,

  Icon: ({ id, size = "1rem", noColor }) => {
    const state = useRead("ListDeployments", {}).data?.find((r) => r.id === id)
      ?.info.state;
    const color = noColor
      ? undefined
      : state && hexColorByIntention(deploymentStateIntention(state));
    return <ICONS.Deployment size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const deployment = useDeployment(id);
    return (
      <ResourceHeader
        type="Deployment"
        id={id}
        resource={deployment}
        intent={deploymentStateIntention(deployment?.info.state)}
        icon={ICONS.Deployment}
        name={deployment?.name}
        state={deployment?.info.state}
        status={deployment?.info.status}
      />
    );
  },

  State: ({ id }) => {
    let state = useDeployment(id)?.info.state;
    return (
      <StatusBadge text={state} intent={deploymentStateIntention(state)} />
    );
  },
  Info: {
    DeployTarget: ({ id }) => {
      const info = useDeployment(id)?.info;
      const swarm = useSwarm(info?.swarm_id);
      const server = useServer(info?.server_id);
      return swarm?.id ? (
        <ResourceLink type="Swarm" id={swarm?.id} />
      ) : server?.id ? (
        <ResourceLink type="Server" id={server?.id} />
      ) : (
        <Group gap="xs">
          <ICONS.Server size="1rem" />
          <Text>Unknown</Text>
        </Group>
      );
    },
    Image: ({ id }) => {
      const config = useFullDeployment(id)?.config;
      const info = useDeployment(id)?.info;
      return info?.build_id ? (
        <ResourceLink type="Build" id={info.build_id} />
      ) : (
        <Group gap="xs">
          <ICONS.Image size="1rem" />
          <Text>
            {info?.image.startsWith("sha256:")
              ? (
                  config?.image as Extract<
                    Types.DeploymentImage,
                    { type: "Image" }
                  >
                )?.params.image
              : info?.image.split("@")[0] || "N/A"}
          </Text>
        </Group>
      );
    },
    DockerResource: ({ id }) => {
      const deployment = useDeployment(id);
      const service = useRead(
        "ListSwarmServices",
        { swarm: deployment?.info.swarm_id! },
        { enabled: !!deployment?.info.swarm_id },
      ).data?.find((service) => service.Name === deployment?.name);
      if (
        !deployment ||
        [
          Types.DeploymentState.Unknown,
          Types.DeploymentState.NotDeployed,
        ].includes(deployment.info.state)
      ) {
        return null;
      }
      if (deployment.info.swarm_id) {
        return (
          <>
            <SwarmResourceLink
              type="Service"
              swarmId={deployment.info.swarm_id}
              resourceId={deployment.name}
              name={deployment.name}
            />
            {service?.Configs.map((config) => (
              <SwarmResourceLink
                key={config}
                type="Config"
                swarmId={deployment.info.swarm_id}
                resourceId={config}
                name={config}
              />
            ))}
            {service?.Secrets.map((secret) => (
              <SwarmResourceLink
                key={secret}
                type="Secret"
                swarmId={deployment.info.swarm_id}
                resourceId={secret}
                name={secret}
              />
            ))}
          </>
        );
      } else {
        return (
          <DockerResourceLink
            type="Container"
            name={deployment.name}
            serverId={deployment.info.server_id}
          />
        );
      }
    },
    Ports: ({ id }) => {
      const deployment = useDeployment(id);
      const container = useRead(
        "ListDockerContainers",
        {
          server: deployment?.info.server_id!,
        },
        { refetchInterval: 10_000, enabled: !!deployment?.info.server_id },
      ).data?.find((container) => container.name === deployment?.name);
      if (!container) return null;
      return (
        <ContainerPorts
          ports={container?.ports ?? []}
          serverId={deployment?.info.server_id}
        />
      );
    },
    UpdateAvailable: DeploymentUpdateAvailable,
  },

  Executions: {
    RunBuild: ({ id }) => {
      const build_id = useDeployment(id)?.info.build_id;
      if (!build_id) return null;
      return <RunBuild id={build_id} />;
    },
    DeployDeployment,
    PullDeployment,
    RestartDeployment,
    PauseUnpauseDeployment,
    StartStopDeployment,
    DestroyDeployment,
  },

  Config: DeploymentTabs,

  Page: {},
};
