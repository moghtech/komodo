import { repoStateIntention, hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "..";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import EntityHeader from "@/ui/entity-header";
import RepoTable from "./table";
import NewResource from "@/resources/new";
import RepoConfig from "./config";

export const RepoComponents: RequiredResourceComponents<
  Types.RepoConfig,
  Types.RepoInfo,
  Types.RepoListItemInfo
> = {
  useList: () => useRead("ListRepos", {}).data,
  useListItem: (id) => RepoComponents.useList()?.find((r) => r.id === id),

  useFull: (id) => useRead("GetRepo", { repo: id }).data,

  useResourceLinks: (repo) => repo?.config?.links,

  useDashboardSummaryData: () => {
    const summary = useRead("GetReposSummary", {}).data;
    return [
      { intention: "Good", value: summary?.ok ?? 0, title: "Ok" },
      {
        intention: "Warning",
        value: (summary?.cloning ?? 0) + (summary?.pulling ?? 0),
        title: "Pulling",
      },
      {
        intention: "Critical",
        value: summary?.failed ?? 0,
        title: "Failed",
      },
      {
        intention: "Unknown",
        value: summary?.unknown ?? 0,
        title: "Unknown",
      },
    ];
  },

  Description: () => <>Configure git repositories.</>,

  New: () => <NewResource type="Repo" />,

  GroupExecutions: () => <></>,

  Table: RepoTable,

  Icon: ({ id, size = "1rem" }) => {
    const state = useRead("ListRepos", {}).data?.find((r) => r.id === id)?.info
      .state;
    const color = state && hexColorByIntention(repoStateIntention(state));
    return <ICONS.Repo size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const repo = RepoComponents.useListItem(id);
    return (
      <EntityHeader
        intent={repoStateIntention(repo?.info.state)}
        icon={<RepoComponents.Icon id={id} size="2rem" />}
        name={repo?.name}
        state={repo?.info.state}
      />
    );
  },

  State: ({ id }) => {
    let state = RepoComponents.useListItem(id)?.info.state;
    return <StatusBadge text={state} intent={repoStateIntention(state)} />;
  },
  Info: {},

  Executions: {},

  Config: RepoConfig,
  DangerZone: ({ id }) => <></>,

  Page: {},
};
