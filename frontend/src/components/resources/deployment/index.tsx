import {
  useExecute,
  useInvalidate,
  usePermissions,
  useRead,
  useWrite,
} from "@lib/hooks";
import { Types } from "komodo_client";
import { RequiredResourceComponents } from "@types";
import { CircleArrowUp, HardDrive, Rocket, Server } from "lucide-react";
import { cn } from "@lib/utils";
import { useServer } from "../server";
import {
  DeployDeployment,
  StartStopDeployment,
  DestroyDeployment,
  RestartDeployment,
  PauseUnpauseDeployment,
  PullDeployment,
} from "./actions";
import {
  deployment_state_intention,
  stroke_color_class_by_intention,
} from "@lib/color";
import { DeploymentTable } from "./table";
import {
  DeleteResource,
  NewResource,
  ResourceLink,
  ResourcePageHeader,
} from "../common";
import { RunBuild } from "../build/actions";
import {
  ActionButton,
  ActionWithDialog,
  DashboardPieChart,
} from "@components/util";
import {
  ContainerPortsTableView,
  DockerResourceLink,
  StatusBadge,
} from "@components/util";
import { GroupActions } from "@components/group-actions";
import { Tooltip, TooltipContent, TooltipTrigger } from "@ui/tooltip";
import { DeploymentTabs } from "./tabs";
import { SwarmResourceLink, useSwarm } from "../swarm";
import { useToast } from "@ui/use-toast";

// const configOrLog = atomWithStorage("config-or-log-v1", "Config");

export const useDeployment = (id?: string) =>
  useRead("ListDeployments", {}, { refetchInterval: 10_000 }).data?.find(
    (d) => d.id === id,
  );

export const useFullDeployment = (id: string) =>
  useRead("GetDeployment", { deployment: id }, { refetchInterval: 10_000 })
    .data;

const DeploymentIcon = ({ id, size }: { id?: string; size: number }) => {
  const state = useDeployment(id)?.info.state;
  const color = stroke_color_class_by_intention(
    deployment_state_intention(state),
  );
  return <Rocket className={cn(`w-${size} h-${size}`, state && color)} />;
};

export const DeploymentComponents: RequiredResourceComponents = {
  list_item: (id) => useDeployment(id),
  resource_links: (resource) =>
    (resource.config as Types.DeploymentConfig).links,

  Description: () => <>Deploy containers on your servers.</>,

  Dashboard: () => {
    const summary = useRead("GetDeploymentsSummary", {}).data;
    const all = [
      summary?.running ?? 0,
      summary?.stopped ?? 0,
      summary?.unhealthy ?? 0,
      summary?.unknown ?? 0,
    ];
    const [running, stopped, unhealthy, unknown] = all;
    return (
      <DashboardPieChart
        data={[
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
        ]}
      />
    );
  },

  New: ({ swarm_id, server_id: _server_id, build_id }) => {
    const swarmsExist = useRead("ListSwarms", {}).data?.length ? true : false;
    const servers = useRead("ListServers", {}).data;
    const server_id = _server_id
      ? _server_id
      : !swarmsExist && servers && servers.length === 1
        ? servers[0].id
        : undefined;
    return (
      <NewResource
        type="Deployment"
        swarm_id={swarm_id}
        selectSwarm={!swarm_id && !server_id}
        server_id={server_id}
        selectServer={!swarm_id && !server_id}
        build_id={build_id}
      />
    );
  },

  Table: ({ resources }) => {
    return (
      <DeploymentTable deployments={resources as Types.DeploymentListItem[]} />
    );
  },

  GroupActions: () => (
    <GroupActions
      type="Deployment"
      actions={[
        "CheckDeploymentForUpdate",
        "PullDeployment",
        "Deploy",
        "RestartDeployment",
        "StopDeployment",
        "DestroyDeployment",
      ]}
    />
  ),

  Icon: ({ id }) => <DeploymentIcon id={id} size={4} />,
  BigIcon: ({ id }) => <DeploymentIcon id={id} size={8} />,

  State: ({ id }) => {
    const state =
      useDeployment(id)?.info.state ?? Types.DeploymentState.Unknown;
    return (
      <StatusBadge text={state} intent={deployment_state_intention(state)} />
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
        <div className="flex gap-2 items-center">
          <Server className="w-4 h-4" />
          <div>Unknown</div>
        </div>
      );
    },
    Image: ({ id }) => {
      const config = useFullDeployment(id)?.config;
      const info = useDeployment(id)?.info;
      return info?.build_id ? (
        <ResourceLink type="Build" id={info.build_id} />
      ) : (
        <div className="flex gap-2 items-center text-sm">
          <HardDrive className="w-4 h-4" />
          <div>
            {info?.image.startsWith("sha256:")
              ? (
                  config?.image as Extract<
                    Types.DeploymentImage,
                    { type: "Image" }
                  >
                )?.params.image
              : info?.image.split("@")[0] || "N/A"}
          </div>
        </div>
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
          <div className="flex items-center gap-x-4 gap-y-2 flex-wrap">
            <SwarmResourceLink
              type="Service"
              swarm_id={deployment.info.swarm_id}
              resource_id={deployment.name}
              name={deployment.name}
            />
            {service?.Configs.map((config) => (
              <div key={config} className="border-l pl-4 text-sm">
                <SwarmResourceLink
                  type="Config"
                  swarm_id={deployment.info.swarm_id}
                  resource_id={config}
                  name={config}
                />
              </div>
            ))}
            {service?.Secrets.map((secret) => (
              <div key={secret} className="border-l pl-4 text-sm">
                <SwarmResourceLink
                  type="Secret"
                  swarm_id={deployment.info.swarm_id}
                  resource_id={secret}
                  name={secret}
                />
              </div>
            ))}
          </div>
        );
      } else {
        return (
          <DockerResourceLink
            type="container"
            name={deployment.name}
            server_id={deployment.info.server_id}
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
        <ContainerPortsTableView
          ports={container?.ports ?? []}
          server_id={deployment?.info.server_id}
        />
      );
    },
  },

  Status: {
    UpdateAvailable: ({ id }) => <UpdateAvailable id={id} />,
  },

  Actions: {
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

  Page: {},

  Config: DeploymentTabs,

  DangerZone: ({ id }) => <DeleteResource type="Deployment" id={id} />,

  ResourcePageHeader: ({ id }) => {
    const deployment = useDeployment(id);

    return (
      <ResourcePageHeader
        intent={deployment_state_intention(deployment?.info.state)}
        icon={<DeploymentIcon id={id} size={8} />}
        type="Deployment"
        id={id}
        resource={deployment}
        state={
          deployment?.info.state === Types.DeploymentState.NotDeployed
            ? "Not Deployed"
            : deployment?.info.state
        }
        status={deployment?.info.status}
      />
    );
  },
};

export const UpdateAvailable = ({
  id,
  small,
}: {
  id: string;
  small?: boolean;
}) => {
  const { toast } = useToast();
  const { canExecute } = usePermissions({ type: "Deployment", id });
  const { mutate: deploy, isPending } = useExecute("Deploy");
  const inv = useInvalidate();
  const { mutate: checkForUpdate, isPending: checkPending } = useWrite(
    "CheckDeploymentForUpdate",
    {
      onSuccess: () => {
        toast({ title: "Checked for updates" });
        inv(["ListDeployments"]);
      },
    },
  );
  const deploying = useRead(
    "GetDeploymentActionState",
    { deployment: id },
    { refetchInterval: 5_000 },
  ).data?.deploying;

  const pending = isPending || deploying;

  const deployment = useDeployment(id);
  const info = deployment?.info;
  const state = info?.state ?? Types.DeploymentState.Unknown;
  if (
    !info ||
    info.swarm_id ||
    info.build_id ||
    [Types.DeploymentState.NotDeployed, Types.DeploymentState.Unknown].includes(
      state,
    )
  ) {
    return null;
  }
  if (small || !canExecute) {
    if (!info?.update_available) {
      return null;
    }
    return (
      <Tooltip>
        <TooltipTrigger asChild>
          <div
            className={cn(
              "px-2 py-1 border rounded-md border-blue-400 hover:border-blue-500 opacity-50 hover:opacity-70 transition-colors cursor-pointer flex items-center gap-2",
              small ? "px-2 py-1" : "px-3 py-2",
            )}
          >
            {!small && (
              <div className="text-sm text-nowrap overflow-hidden overflow-ellipsis">
                Update Available
              </div>
            )}
            <CircleArrowUp className="w-4 h-4" />
          </div>
        </TooltipTrigger>
        <TooltipContent className="w-fit text-sm">
          There is a newer image available.
        </TooltipContent>
      </Tooltip>
    );
  }

  if (!info?.update_available) {
    return (
      <ActionButton
        variant="outline"
        className="flex gap-2 items-center opacity-50 max-w-fit"
        onClick={() => checkForUpdate({ deployment: id })}
        loading={checkPending}
        title="Check"
        icon={<CircleArrowUp className="w-4 h-4" />}
      />
    );
  }

  return (
    <ActionWithDialog
      name={deployment.name}
      variant="outline"
      targetClassName="border-blue-400 hover:border-blue-500 opacity-50 hover:opacity-70 max-w-fit"
      title="Update Available"
      openTitle="Redeploy"
      icon={<CircleArrowUp className="w-4 h-4" />}
      onClick={() => deploy({ deployment: id })}
      disabled={pending}
      loading={pending}
    />
  );
};
