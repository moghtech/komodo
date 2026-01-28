import { Types } from "komodo_client";
import { ActionComponents } from "./action";
import { PieChartItem } from "@/components/dashboard-summary";
import React from "react";
import { ServerComponents } from "./server";
import { StackComponents } from "./stack";
import { SwarmComponents } from "./swarm";
import { DeploymentComponents } from "./deployment";
import { BuildComponents } from "./build";
import { RepoComponents } from "./repo";
import { ProcedureComponents } from "./procedure";
import { ResourceSyncComponents } from "./sync";
import { BuilderComponents } from "./builder";
import { AlerterComponents } from "./alerter";
import { BoxProps, TableProps } from "@mantine/core";

export type UsableResourceTarget = Exclude<
  Types.ResourceTarget,
  { type: "System" }
>;
export type UsableResource = Exclude<Types.ResourceTarget["type"], "System">;

export const RESOURCE_TARGETS: UsableResource[] = [
  "Swarm",
  "Server",
  "Stack",
  "Deployment",
  "Build",
  "Repo",
  "Procedure",
  "Action",
  "Builder",
  "Alerter",
  "ResourceSync",
];

export const SETTINGS_RESOURCES: UsableResource[] = ["Builder", "Alerter"];

export const SIDEBAR_RESOURCES: UsableResource[] = RESOURCE_TARGETS.filter(
  (target) => !SETTINGS_RESOURCES.includes(target),
);

export const ResourceComponents: {
  [key in UsableResource]: RequiredResourceComponents;
} = {
  Swarm: SwarmComponents,
  Server: ServerComponents,
  Stack: StackComponents,
  Deployment: DeploymentComponents,
  Build: BuildComponents,
  Repo: RepoComponents,
  Procedure: ProcedureComponents,
  Action: ActionComponents,
  ResourceSync: ResourceSyncComponents,
  Builder: BuilderComponents,
  Alerter: AlerterComponents,
};

type IdComponent = React.FC<{ id: string }>;

export interface RequiredResourceComponents<
  Config = any,
  Info = any,
  ListItemInfo = any,
> {
  useListItem: (id: string) => Types.ResourceListItem<ListItemInfo> | undefined;
  useFull: (id: string) => Types.Resource<Config, Info> | undefined;
  useResourceLinks: (
    resource: Types.Resource<Config, Info> | undefined,
  ) => Array<string> | undefined;
  useDashboardSummaryData?: () => Array<PieChartItem | false | undefined>;

  Description: React.FC;

  /** Header for individual resource pages */
  ResourcePageHeader: IdComponent;

  /** Alternate summary card for use in dashboard */
  DashboardSummary?: React.FC;

  /** New resource button / dialog */
  New: React.FC<{
    swarm_id?: string;
    server_id?: string;
    builder_id?: string;
    build_id?: string;
  }>;

  /** A table component to view resource list */
  Table: React.FC<
    {
      resources: Types.ResourceListItem<ListItemInfo>[];
      tableProps?: TableProps;
    } & BoxProps
  >;

  /** Dropdown menu to trigger group executions for selected resources */
  GroupExecutions: React.FC;

  /** Icon for the resource */
  Icon: React.FC<{ id?: string; size?: string | number }>;

  State: IdComponent;

  /** Config component for resource */
  Config: IdComponent;

  /** Danger zone for resource, containing eg delete */
  DangerZone: IdComponent;

  /** status metrics, like deployment state / status */
  Status: { [status: string]: IdComponent };

  /**
   * Some config items shown in header, like deployment server /image
   * or build repo / branch
   */
  Info: { [info: string]: IdComponent };

  /** Execution buttons */
  Executions: { [action: string]: IdComponent };

  /** Resource specific sections */
  Page: { [section: string]: IdComponent };
}
