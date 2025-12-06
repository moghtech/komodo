import { useRead } from "@lib/hooks";
import { RequiredResourceComponents } from "@types";
import {
  Component,
  Diamond,
  FolderCode,
  KeyRound,
  ListTodo,
  Settings,
  SquareStack,
} from "lucide-react";
import { DeleteResource, NewResource, ResourcePageHeader } from "../common";
import { SwarmTable } from "./table";
import {
  swarm_state_intention,
  stroke_color_class_by_intention,
} from "@lib/color";
import { cn, updateLogToHtml } from "@lib/utils";
import { Types } from "komodo_client";
import { DashboardPieChart } from "@components/util";
import { StatusBadge } from "@components/util";
import { GroupActions } from "@components/group-actions";
import { Tooltip, TooltipContent, TooltipTrigger } from "@ui/tooltip";
import { Card } from "@ui/card";
import { SwarmTabs } from "./tabs";
import { Link } from "react-router-dom";

export const useSwarm = (id?: string) =>
  useRead("ListSwarms", {}, { refetchInterval: 10_000 }).data?.find(
    (d) => d.id === id
  );

export const useFullSwarm = (id: string) =>
  useRead("GetSwarm", { swarm: id }, { refetchInterval: 10_000 }).data;

const SwarmIcon = ({ id, size }: { id?: string; size: number }) => {
  const state = useSwarm(id)?.info.state;
  const color = stroke_color_class_by_intention(swarm_state_intention(state));
  return <Component className={cn(`w-${size} h-${size}`, state && color)} />;
};

export const SwarmComponents: RequiredResourceComponents = {
  list_item: (id) => useSwarm(id),
  resource_links: (resource) => (resource.config as Types.SwarmConfig).links,

  Description: () => <>Control and monitor docker swarms.</>,

  Dashboard: () => {
    const summary = useRead("GetSwarmsSummary", {}).data;
    return (
      <DashboardPieChart
        data={[
          { intention: "Good", value: summary?.healthy ?? 0, title: "Healthy" },
          {
            intention: "Critical",
            value: summary?.unhealthy ?? 0,
            title: "Unhealthy",
          },
          {
            intention: "Unknown",
            value: summary?.unknown ?? 0,
            title: "Unknown",
          },
        ]}
      />
    );
  },

  New: () => <NewResource type="Swarm" />,

  GroupActions: () => <GroupActions type="Swarm" actions={[]} />,

  Table: ({ resources }) => (
    <SwarmTable swarms={resources as Types.SwarmListItem[]} />
  ),

  Icon: ({ id }) => <SwarmIcon id={id} size={4} />,
  BigIcon: ({ id }) => <SwarmIcon id={id} size={8} />,

  State: ({ id }) => {
    const state = useSwarm(id)?.info.state;
    return <StatusBadge text={state} intent={swarm_state_intention(state)} />;
  },

  Info: {},

  Status: {
    Err: ({ id }) => {
      const err = useSwarm(id)?.info.err;
      if (!err) return null;
      return (
        <Tooltip>
          <TooltipTrigger asChild>
            <Card className="px-3 py-2 bg-destructive/75 hover:bg-destructive transition-colors cursor-pointer">
              <div className="text-sm text-nowrap overflow-hidden overflow-ellipsis">
                Error
              </div>
            </Card>
          </TooltipTrigger>
          <TooltipContent className="w-fit max-w-[90vw] md:max-w-[60vw]">
            <pre
              dangerouslySetInnerHTML={{
                __html: updateLogToHtml(err),
              }}
            />
          </TooltipContent>
        </Tooltip>
      );
    },
  },

  Actions: {},

  Page: {},

  Config: SwarmTabs,

  DangerZone: ({ id }) => <DeleteResource type="Swarm" id={id} />,

  ResourcePageHeader: ({ id }) => {
    const swarm = useSwarm(id);

    return (
      <ResourcePageHeader
        intent={swarm_state_intention(swarm?.info.state)}
        icon={<SwarmIcon id={id} size={8} />}
        type="Swarm"
        id={id}
        resource={swarm}
        state={swarm?.info.state}
        status=""
      />
    );
  },
};

export type SwarmResourceType =
  | "Node"
  | "Service"
  | "Task"
  | "Secret"
  | "Config"
  | "Stack";

export const SWARM_ICONS: {
  [type in SwarmResourceType]: React.FC<{ size?: number; className?: string }>;
} = {
  Node: ({ size, className }) => (
    <Diamond className={cn(`w-${size} h-${size}`, className)} />
  ),
  Service: ({ size, className }) => (
    <FolderCode className={cn(`w-${size} h-${size}`, className)} />
  ),
  Task: ({ size, className }) => (
    <ListTodo className={cn(`w-${size} h-${size}`, className)} />
  ),
  Secret: ({ size, className }) => (
    <KeyRound className={cn(`w-${size} h-${size}`, className)} />
  ),
  Config: ({ size, className }) => (
    <Settings className={cn(`w-${size} h-${size}`, className)} />
  ),
  Stack: ({ size, className }) => (
    <SquareStack className={cn(`w-${size} h-${size}`, className)} />
  ),
};

export const SwarmResourceLink = ({
  type,
  swarm_id,
  resource_id,
  name,
}: {
  type: SwarmResourceType;
  swarm_id: string;
  resource_id: string | undefined;
  name: string | undefined;
}) => {
  const Icon = SWARM_ICONS[type];
  return (
    <Link
      to={`/swarms/${swarm_id}/swarm-${type.toLowerCase()}/${resource_id}`}
      className="flex gap-2 items-center hover:underline"
    >
      <Icon size={4} />
      {name ?? "Unknown"}
    </Link>
  );
};
