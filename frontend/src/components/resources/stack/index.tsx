import { useInvalidate, useRead, useWrite } from "@lib/hooks";
import { RequiredResourceComponents } from "@types";
import { Card } from "@ui/card";
import {
  CircleArrowUp,
  Layers,
  Loader2,
  RefreshCcw,
  Server,
} from "lucide-react";
import {
  DeleteResource,
  NewResource,
  ResourceLink,
  ResourcePageHeader,
  StandardSource,
} from "../common";
import { StackTable } from "./table";
import {
  border_color_class_by_intention,
  stack_state_intention,
  stroke_color_class_by_intention,
} from "@lib/color";
import { cn } from "@lib/utils";
import { useServer } from "../server";
import { Types } from "komodo_client";
import {
  DeployStack,
  DestroyStack,
  PauseUnpauseStack,
  PullStack,
  RestartStack,
  StartStopStack,
} from "./actions";
import { Badge } from "@ui/badge";
import { Button } from "@ui/button";
import { useToast } from "@ui/use-toast";
import { DashboardPieChart } from "@components/util";
import { StatusBadge } from "@components/util";
import { GroupActions } from "@components/group-actions";
import { Tooltip, TooltipTrigger, TooltipContent } from "@ui/tooltip";
import { StackTabs } from "./tabs";
import { useSwarm } from "../swarm";

export const useStack = (id?: string) =>
  useRead("ListStacks", {}, { refetchInterval: 10_000 }).data?.find(
    (d) => d.id === id
  );

export const useFullStack = (id: string) =>
  useRead("GetStack", { stack: id }, { refetchInterval: 10_000 }).data;

const StackIcon = ({ id, size }: { id?: string; size: number }) => {
  const state = useStack(id)?.info.state;
  const color = stroke_color_class_by_intention(stack_state_intention(state));
  return <Layers className={cn(`w-${size} h-${size}`, state && color)} />;
};

export const StackComponents: RequiredResourceComponents = {
  list_item: (id) => useStack(id),
  resource_links: (resource) => (resource.config as Types.StackConfig).links,

  Description: () => <>Deploy docker compose files.</>,

  Dashboard: () => {
    const summary = useRead("GetStacksSummary", {}).data;
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
            title: "Down",
            intention: "Neutral",
            value: summary?.down ?? 0,
          },
          { intention: "Good", value: running, title: "Running" },
          {
            intention: "Warning",
            value: stopped,
            title: "Stopped",
          },
          {
            intention: "Critical",
            value: unhealthy,
            title: "Unhealthy",
          },
          {
            intention: "Unknown",
            value: unknown,
            title: "Unknown",
          },
        ]}
      />
    );
  },

  GroupActions: () => (
    <GroupActions
      type="Stack"
      actions={[
        "PullStack",
        "DeployStack",
        "RestartStack",
        "StopStack",
        "DestroyStack",
      ]}
    />
  ),

  New: ({ server_id: _server_id }) => {
    const servers = useRead("ListServers", {}).data;
    const server_id = _server_id
      ? _server_id
      : servers && servers.length === 1
        ? servers[0].id
        : undefined;
    return (
      <NewResource
        type="Stack"
        server_id={server_id}
        selectServer={!server_id}
      />
    );
  },

  Table: ({ resources }) => (
    <StackTable stacks={resources as Types.StackListItem[]} />
  ),

  Icon: ({ id }) => <StackIcon id={id} size={4} />,
  BigIcon: ({ id }) => <StackIcon id={id} size={8} />,

  State: ({ id }) => {
    const state = useStack(id)?.info.state ?? Types.StackState.Unknown;
    return <StatusBadge text={state} intent={stack_state_intention(state)} />;
  },

  Info: {
    Server: ({ id }) => {
      const info = useStack(id)?.info;
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
    Source: ({ id }) => {
      const info = useStack(id)?.info;
      return <StandardSource info={info} />;
    },
    // Branch: ({ id }) => {
    //   const config = useFullStack(id)?.config;
    //   const file_contents = config?.file_contents;
    //   if (file_contents || !config?.branch) return null;
    //   return (
    //     <div className="flex items-center gap-2">
    //       <GitBranch className="w-4 h-4" />
    //       {config.branch}
    //     </div>
    //   );
    // },
    Services: ({ id }) => {
      const info = useStack(id)?.info;
      return (
        <div className="flex gap-1">
          <div className="font-bold">{info?.services.length}</div>
          <div>Service{(info?.services.length ?? 0 > 1) ? "s" : ""}</div>
        </div>
      );
    },
  },

  Status: {
    NoConfig: ({ id }) => {
      const config = useFullStack(id)?.config;
      if (
        !config ||
        config?.files_on_host ||
        config?.file_contents ||
        config?.linked_repo ||
        config?.repo
      ) {
        return null;
      }
      return (
        <Tooltip>
          <TooltipTrigger asChild>
            <Card className="px-3 py-2 bg-destructive/75 hover:bg-destructive transition-colors cursor-pointer">
              <div className="text-sm text-nowrap overflow-hidden overflow-ellipsis">
                Config Missing
              </div>
            </Card>
          </TooltipTrigger>
          <TooltipContent>
            <div className="grid gap-2">
              No configuration provided for stack. Cannot get stack state.
              Either paste the compose file contents into the UI, or configure a
              git repo containing your files.
            </div>
          </TooltipContent>
        </Tooltip>
      );
    },
    ProjectMissing: ({ id }) => {
      const info = useStack(id)?.info;
      const state = info?.state ?? Types.StackState.Unknown;
      if (
        !info ||
        !info?.project_missing ||
        [
          Types.StackState.Deploying,
          Types.StackState.Down,
          Types.StackState.Unknown,
        ].includes(state)
      ) {
        return null;
      }
      return (
        <Tooltip>
          <TooltipTrigger asChild>
            <Card className="px-3 py-2 bg-destructive/75 hover:bg-destructive transition-colors cursor-pointer">
              <div className="text-sm text-nowrap overflow-hidden overflow-ellipsis">
                Project Missing
              </div>
            </Card>
          </TooltipTrigger>
          <TooltipContent>
            <div className="grid gap-2">
              The compose project is not on the host. If the compose stack is
              running, the 'Project Name' needs to be set. This can be found
              with 'docker compose ls'.
            </div>
          </TooltipContent>
        </Tooltip>
      );
    },
    RemoteErrors: ({ id }) => {
      const info = useFullStack(id)?.info;
      const errors = info?.remote_errors;
      if (!info || !errors || errors.length === 0) {
        return null;
      }
      return (
        <Tooltip>
          <TooltipTrigger asChild>
            <Card className="px-3 py-2 bg-destructive/75 hover:bg-destructive transition-colors cursor-pointer">
              <div className="text-sm text-nowrap overflow-hidden overflow-ellipsis">
                Remote Error
              </div>
            </Card>
          </TooltipTrigger>
          <TooltipContent>
            <div>
              There are errors reading the remote file contents. See{" "}
              <span className="font-bold">Info</span> tab for details.
            </div>
          </TooltipContent>
        </Tooltip>
      );
    },
    UpdateAvailable: ({ id }) => <UpdateAvailable id={id} />,
    Hash: ({ id }) => {
      const info = useStack(id)?.info;
      const fullInfo = useFullStack(id)?.info;
      const state = info?.state;
      const stackDown =
        state === undefined ||
        state === Types.StackState.Unknown ||
        state === Types.StackState.Down;
      if (
        stackDown ||
        info?.project_missing ||
        !info?.latest_hash ||
        !fullInfo
      ) {
        return null;
      }
      const out_of_date =
        info.deployed_hash && info.deployed_hash !== info.latest_hash;
      return (
        <Tooltip>
          <TooltipTrigger asChild>
            <Card
              className={cn(
                "px-3 py-2 hover:bg-accent/50 transition-colors cursor-pointer",
                out_of_date && border_color_class_by_intention("Warning")
              )}
            >
              <div className="text-muted-foreground text-sm text-nowrap overflow-hidden overflow-ellipsis">
                {info.deployed_hash ? "deployed" : "latest"}:{" "}
                {info.deployed_hash || info.latest_hash}
              </div>
            </Card>
          </TooltipTrigger>
          <TooltipContent>
            <div className="grid gap-2">
              <Badge
                variant="secondary"
                className="w-fit text-muted-foreground"
              >
                message
              </Badge>
              {fullInfo.deployed_message || fullInfo.latest_message}
              {out_of_date && (
                <>
                  <Badge
                    variant="secondary"
                    className={cn(
                      "w-fit text-muted-foreground border-[1px]",
                      border_color_class_by_intention("Warning")
                    )}
                  >
                    latest
                  </Badge>
                  <div>
                    <span className="text-muted-foreground">
                      {info.latest_hash}
                    </span>
                    : {fullInfo.latest_message}
                  </div>
                </>
              )}
            </div>
          </TooltipContent>
        </Tooltip>
      );
    },
    Refresh: ({ id }) => {
      const { toast } = useToast();
      const inv = useInvalidate();
      const { mutate, isPending } = useWrite("RefreshStackCache", {
        onSuccess: () => {
          inv(["ListStacks"], ["GetStack", { stack: id }]);
          toast({ title: "Refreshed stack status cache" });
        },
      });
      return (
        <Button
          variant="outline"
          size="icon"
          onClick={() => {
            mutate({ stack: id });
            toast({ title: "Triggered refresh of stack status cache" });
          }}
        >
          {isPending ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            <RefreshCcw className="w-4 h-4" />
          )}
        </Button>
      );
    },
  },

  Actions: {
    DeployStack,
    PullStack,
    RestartStack,
    PauseUnpauseStack,
    StartStopStack,
    DestroyStack,
  },

  Page: {},

  Config: StackTabs,

  DangerZone: ({ id }) => <DeleteResource type="Stack" id={id} />,

  ResourcePageHeader: ({ id }) => {
    const stack = useStack(id);
    return (
      <ResourcePageHeader
        intent={stack_state_intention(stack?.info.state)}
        icon={<StackIcon id={id} size={8} />}
        type="Stack"
        id={id}
        resource={stack}
        state={stack?.info.state}
        status={
          stack?.info.state === Types.StackState.Unhealthy
            ? stack?.info.status
            : undefined
        }
      />
    );
  },
};

export const UpdateAvailable = ({
  id,
  small = false,
}: {
  id: string;
  small?: boolean;
}) => {
  const info = useStack(id)?.info;
  const state = info?.state ?? Types.StackState.Unknown;
  if (
    !info ||
    !!info?.services.every((service) => !service.update_available) ||
    [Types.StackState.Down, Types.StackState.Unknown].includes(state)
  ) {
    return null;
  }
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <div
          className={cn(
            "px-2 py-1 border rounded-md border-blue-400 hover:border-blue-500 opacity-50 hover:opacity-70 transition-colors cursor-pointer flex items-center gap-2",
            small ? "px-2 py-1" : "px-3 py-2"
          )}
        >
          <CircleArrowUp className="w-4 h-4" />
          {!small && (
            <div className="text-sm text-nowrap overflow-hidden overflow-ellipsis">
              Update
              {(info?.services.filter((s) => s.update_available).length ?? 0) >
              1
                ? "s"
                : ""}{" "}
              Available
            </div>
          )}
        </div>
      </TooltipTrigger>
      <TooltipContent className="flex flex-col gap-2 w-fit">
        {info?.services
          .filter((service) => service.update_available)
          .map((s) => (
            <div className="text-sm flex gap-2">
              <div className="text-muted-foreground">{s.service}</div>
              <div className="text-muted-foreground"> - </div>
              <div>{s.image}</div>
            </div>
          ))}
      </TooltipContent>
    </Tooltip>
  );
};
