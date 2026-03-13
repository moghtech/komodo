import { repoStateIntention, hexColorByIntention } from "@/lib/color";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { RequiredResourceComponents } from "@/resources";
import { Types } from "komodo_client";
import StatusBadge from "@/ui/status-badge";
import RepoTable from "./table";
import NewResource from "@/resources/new";
import ResourceHeader from "@/resources/header";
import RepoTabs from "./tabs";
import BatchExecutions from "@/components/batch-executions";
import { BuildRepo, CloneRepo, PullRepo } from "./executions";

export function useRepo(id: string | undefined) {
  return useRead("ListRepos", {}).data?.find((r) => r.id === id);
}

export function useFullRepo(id: string) {
  return useRead("GetRepo", { repo: id }).data;
}

export const RepoComponents: RequiredResourceComponents<
  Types.RepoConfig,
  Types.RepoInfo,
  Types.RepoListItemInfo
> = {
  useList: () => useRead("ListRepos", {}).data,
  useListItem: useRepo,
  useFull: useFullRepo,

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

  BatchExecutions: () => (
    <BatchExecutions
      type="Repo"
      executions={[
        ["PullRepo", ICONS.PullRepo],
        ["CloneRepo", ICONS.CloneRepo],
        ["BuildRepo", ICONS.Build],
      ]}
    />
  ),

  Table: RepoTable,

  Icon: ({ id, size = "1rem", noColor }) => {
    const state = useRead("ListRepos", {}).data?.find((r) => r.id === id)?.info
      .state;
    const color = noColor
      ? undefined
      : state && hexColorByIntention(repoStateIntention(state));
    return <ICONS.Repo size={size} color={color} />;
  },

  ResourcePageHeader: ({ id }) => {
    const repo = useRepo(id);
    return (
      <ResourceHeader
        type="Repo"
        id={id}
        resource={repo}
        intent={repoStateIntention(repo?.info.state)}
        icon={ICONS.Repo}
        name={repo?.name}
        state={repo?.info.state}
      />
    );
  },

  State: ({ id }) => {
    let state = useRepo(id)?.info.state;
    return <StatusBadge text={state} intent={repoStateIntention(state)} />;
  },
  Info: {},

  Executions: {
    CloneRepo,
    PullRepo,
    BuildRepo,
  },

  Config: RepoTabs,

  Page: {},
};
