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
import { useServer } from "../server";
import { useBuilder } from "../builder";
import { Box, Group } from "@mantine/core";
import ResourceLink from "../link";
import RepoLink from "@/components/repo-link";

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
  Info: {
    Target: ({ id }) => {
      const info = useRepo(id)?.info;
      const server = useServer(info?.server_id);
      const builder = useBuilder(info?.builder_id);

      if (!info?.server_id && !info?.builder_id) return null;

      return (
        <>
          {server?.id && (
            <Box>
              <ResourceLink type="Server" id={server.id} />
            </Box>
          )}
          {builder?.id && (
            <Box>
              <ResourceLink type="Builder" id={builder.id} />
            </Box>
          )}
        </>
      );
    },
    Source: ({ id }) => {
      const info = useRepo(id)?.info;

      if (!info?.repo || !info?.repo_link) return null;

      return <RepoLink repo={info?.repo} link={info?.repo_link} />;
    },
    Branch: ({ id }) => {
      const branch = useRepo(id)?.info.branch;
      return (
        <Group gap="xs" wrap="nowrap">
          <ICONS.Branch size="1rem" />
          {branch}
        </Group>
      );
    },
  },

  Executions: {
    CloneRepo,
    PullRepo,
    BuildRepo,
  },

  Config: RepoTabs,

  Page: {},
};
