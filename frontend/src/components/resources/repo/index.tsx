import { useInvalidate, useRead, useWrite } from "@lib/hooks";
import { RequiredResourceComponents } from "@types";
import { Card } from "@ui/card";
import { GitBranch, Loader2, RefreshCcw } from "lucide-react";
import { RepoConfig } from "./config";
import { BuildRepo, CloneRepo, PullRepo } from "./actions";
import { DeleteResource, NewResource, ResourceLink } from "../common";
import { RepoTable } from "./table";
import {
  repo_state_intention,
  stroke_color_class_by_intention,
} from "@lib/color";
import { cn } from "@lib/utils";
import { useServer } from "../server";
import { Types } from "komodo_client";
import { DashboardPieChart } from "@pages/home/dashboard";
import { RepoLink, ResourcePageHeader, StatusBadge } from "@components/util";
import { Badge } from "@ui/badge";
import { useToast } from "@ui/use-toast";
import { Button } from "@ui/button";
import { useBuilder } from "../builder";
import { GroupActions } from "@components/group-actions";
import { Tooltip, TooltipContent, TooltipTrigger } from "@ui/tooltip";

export const useRepo = (id?: string) =>
  useRead("ListRepos", {}, { refetchInterval: 10_000 }).data?.find(
    (d) => d.id === id
  );

export const useFullRepo = (id: string) =>
  useRead("GetRepo", { repo: id }, { refetchInterval: 10_000 }).data;

const RepoIcon = ({ id, size }: { id?: string; size: number }) => {
  const state = useRepo(id)?.info.state;
  const color = stroke_color_class_by_intention(repo_state_intention(state));
  return <GitBranch className={cn(`w-${size} h-${size}`, state && color)} />;
};

export const RepoComponents: RequiredResourceComponents = {
  list_item: (id) => useRepo(id),
  resource_links: (resource) => (resource.config as Types.RepoConfig).links,

  Description: () => <>Build using custom scripts. Or anything else.</>,

  Dashboard: () => {
    const summary = useRead("GetReposSummary", {}).data;
    return (
      <DashboardPieChart
        data={[
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
        ]}
      />
    );
  },

  New: ({ server_id }) => <NewResource type="Repo" server_id={server_id} />,

  GroupActions: () => (
    <GroupActions
      type="Repo"
      actions={["PullRepo", "CloneRepo", "BuildRepo"]}
    />
  ),

  Table: ({ resources }) => (
    <RepoTable repos={resources as Types.RepoListItem[]} />
  ),

  Icon: ({ id }) => <RepoIcon id={id} size={4} />,
  BigIcon: ({ id }) => <RepoIcon id={id} size={8} />,

  State: ({ id }) => {
    const state = useRepo(id)?.info.state;
    return <StatusBadge text={state} intent={repo_state_intention(state)} />;
  },

  Info: {
    Target: ({ id }) => {
      const info = useRepo(id)?.info;
      const server = useServer(info?.server_id);
      const builder = useBuilder(info?.builder_id);
      return (
        <div className="flex items-center gap-x-4 gap-y-2 flex-wrap">
          {server?.id &&
            (builder?.id ? (
              <div className="pr-4 text-sm border-r">
                <ResourceLink type="Server" id={server.id} />
              </div>
            ) : (
              <ResourceLink type="Server" id={server.id} />
            ))}
          {builder?.id && <ResourceLink type="Builder" id={builder.id} />}
        </div>
      );
    },
    Source: ({ id }) => {
      const info = useRepo(id)?.info;
      if (!info) {
        return <Loader2 className="w-4 h-4 animate-spin" />;
      }
      return <RepoLink link={info.repo_link} repo={info.repo} />;
    },
    Branch: ({ id }) => {
      const branch = useRepo(id)?.info.branch;
      return (
        <div className="flex items-center gap-2">
          <GitBranch className="w-4 h-4" />
          {branch}
        </div>
      );
    },
  },

  Status: {
    Cloned: ({ id }) => {
      const info = useRepo(id)?.info;
      if (!info?.cloned_hash || info.cloned_hash === info.latest_hash) {
        return null;
      }
      return (
        <Tooltip>
          <TooltipTrigger asChild>
            <Card className="px-3 py-2 hover:bg-accent/50 transition-colors cursor-pointer">
              <div className="text-muted-foreground text-sm text-nowrap overflow-hidden overflow-ellipsis">
                cloned: {info.cloned_hash}
              </div>
            </Card>
          </TooltipTrigger>
          <TooltipContent>
            <div className="grid">
              <div className="text-muted-foreground">commit message:</div>
              {info.cloned_message}
            </div>
          </TooltipContent>
        </Tooltip>
      );
    },
    Built: ({ id }) => {
      const info = useRepo(id)?.info;
      const fullInfo = useFullRepo(id)?.info;
      if (!info?.built_hash || info.built_hash === info.latest_hash) {
        return null;
      }
      return (
        <Tooltip>
          <TooltipTrigger asChild>
            <Card className="px-3 py-2 hover:bg-accent/50 transition-colors cursor-pointer">
              <div className="text-muted-foreground text-sm text-nowrap overflow-hidden overflow-ellipsis">
                built: {info.built_hash}
              </div>
            </Card>
          </TooltipTrigger>
          <TooltipContent>
            <div className="grid gap-2">
              <Badge
                variant="secondary"
                className="w-fit text-muted-foreground"
              >
                commit message
              </Badge>
              {fullInfo?.built_message}
            </div>
          </TooltipContent>
        </Tooltip>
      );
    },
    Latest: ({ id }) => {
      const info = useRepo(id)?.info;
      const fullInfo = useFullRepo(id)?.info;
      if (!info?.latest_hash) {
        return null;
      }
      return (
        <Tooltip>
          <TooltipTrigger asChild>
            <Card className="px-3 py-2 hover:bg-accent/50 transition-colors cursor-pointer">
              <div className="text-muted-foreground text-sm text-nowrap overflow-hidden overflow-ellipsis">
                latest: {info.latest_hash}
              </div>
            </Card>
          </TooltipTrigger>
          <TooltipContent>
            <div className="grid gap-2">
              <Badge
                variant="secondary"
                className="w-fit text-muted-foreground"
              >
                commit message
              </Badge>
              {fullInfo?.latest_message}
            </div>
          </TooltipContent>
        </Tooltip>
      );
    },
    Refresh: ({ id }) => {
      const { toast } = useToast();
      const inv = useInvalidate();
      const { mutate, isPending } = useWrite("RefreshRepoCache", {
        onSuccess: () => {
          inv(["ListRepos"], ["GetRepo", { repo: id }]);
          toast({ title: "Refreshed repo status cache" });
        },
      });
      return (
        <Button
          variant="outline"
          size="icon"
          onClick={() => {
            mutate({ repo: id });
            toast({ title: "Triggered refresh of repo status cache" });
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

  Actions: { BuildRepo, PullRepo, CloneRepo },

  Page: {},

  Config: RepoConfig,

  DangerZone: ({ id }) => <DeleteResource type="Repo" id={id} />,

  ResourcePageHeader: ({ id }) => {
    const repo = useRepo(id);

    return (
      <ResourcePageHeader
        intent={repo_state_intention(repo?.info.state)}
        icon={<RepoIcon id={id} size={8} />}
        type="Repo"
        id={id}
        name={repo?.name}
        state={repo?.info.state}
        status=""
      />
    );
  },
};
