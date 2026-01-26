import { Types } from "komodo_client";
import { ActionComponents } from "./action";
import { PieChartItem } from "@/components/dashboard-summary";
import React from "react";

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
  Swarm: ActionComponents,
  Server: ActionComponents,
  Stack: ActionComponents,
  Deployment: ActionComponents,
  Build: ActionComponents,
  Repo: ActionComponents,
  Procedure: ActionComponents,
  Action: ActionComponents,
  ResourceSync: ActionComponents,
  Builder: ActionComponents,
  Alerter: ActionComponents,
};

type IdComponent = React.FC<{ id: string }>;

export interface RequiredResourceComponents {
  useListItem: (id: string) => Types.ResourceListItem<unknown> | undefined;
  useFull: (id: string) => Types.Resource<unknown, unknown> | undefined;
  useResourceLinks: (
    resource: Types.Resource<unknown, unknown> | undefined,
  ) => Array<string> | undefined;
  useDashboardSummaryData?: () => Array<PieChartItem>;

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
  Table: React.FC<{ resources: Types.ResourceListItem<unknown>[] }>;

  /** Dropdown menu to trigger group actions for selected resources */
  GroupActions: React.FC;

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

  /** Action buttons */
  Actions: { [action: string]: IdComponent };

  /** Resource specific sections */
  Page: { [section: string]: IdComponent };
}
