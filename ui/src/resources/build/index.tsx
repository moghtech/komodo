import { buildStateIntention, hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import EntityHeader from "@/ui/entity-header";
import BuildTable from "./table";
import NewResource from "@/resources/new";
import BuildConfig from "./config";

export const BuildComponents: RequiredResourceComponents<
  Types.BuildConfig,
  Types.BuildInfo,
  Types.BuildListItemInfo
> = {
  useList: () => useRead("ListBuilds", {}).data,
  useListItem: (id) => BuildComponents.useList()?.find((r) => r.id === id),

  useFull: (id) => useRead("GetBuild", { build: id }).data,

  useResourceLinks: (build) => build?.config?.links,

  useDashboardSummaryData: () => {
    const summary = useRead("GetBuildsSummary", {}).data;
    return [
      { title: "Ok", intention: "Good", value: summary?.ok ?? 0 },
      {
        title: "Building",
        intention: "Warning",
        value: summary?.building ?? 0,
      },
      {
        title: "Failed",
        intention: "Critical",
        value: summary?.failed ?? 0,
      },
      {
        title: "Unknown",
        intention: "Unknown",
        value: summary?.unknown ?? 0,
      },
    ];
  },

  Description: () => <>Build container images.</>,

  New: () => <NewResource type="Build" />,

  GroupExecutions: () => <></>,

  Table: BuildTable,

  Icon: ({ id, size = "1rem" }) => {
    const state = useRead("ListBuilds", {}).data?.find((r) => r.id === id)?.info
      .state;
    const color = state && hexColorByIntention(buildStateIntention(state));
    return <ICONS.Build size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const build = BuildComponents.useListItem(id);
    return (
      <EntityHeader
        intent={buildStateIntention(build?.info.state)}
        icon={ICONS.Build}
        name={build?.name}
        state={build?.info.state}
      />
    );
  },

  State: ({ id }) => {
    let state = BuildComponents.useListItem(id)?.info.state;
    return <StatusBadge text={state} intent={buildStateIntention(state)} />;
  },
  Info: {},

  Executions: {},

  Config: BuildConfig,
  DangerZone: ({ id }) => <></>,

  Page: {},
};
