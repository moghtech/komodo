import { ExportButton } from "@components/export";
import { Page, Section } from "@components/layouts";
import { ResourceComponents } from "@components/resources";
import { ResourceLink, ResourceNameSimple } from "@components/resources/common";
import { ServerStatsMini } from "@components/resources/server";
import { TagsWithBadge } from "@components/tags";
import { StatusBadge, TemplateMarker } from "@components/util";
import { useDashboardPreferences } from "@lib/dashboard-preferences";
import { Button } from "@ui/button";
import { Eye, EyeOff } from "lucide-react";
import {
  action_state_intention,
  build_state_intention,
  ColorIntention,
  hex_color_by_intention,
  procedure_state_intention,
  repo_state_intention,
  text_color_class_by_intention,
} from "@lib/color";
import { useNoResources, useRead, useUser } from "@lib/hooks";
import { cn, usableResourcePath } from "@lib/utils";
import { Types } from "komodo_client";
import { UsableResource } from "@types";
import { DataTable, SortableHeader } from "@ui/data-table";
import { AlertTriangle, Box, Circle, History } from "lucide-react";
import { PieChart } from "react-minimal-pie-chart";
import { Link } from "react-router-dom";
import { UpdateAvailable as StackUpdateAvailable } from "@components/resources/stack";
import { UpdateAvailable as DeploymentUpdateAvailable } from "@components/resources/deployment";

export default function Dashboard() {
  const noResources = useNoResources();
  const user = useUser().data!;
  const { preferences, updatePreference } = useDashboardPreferences();

  return (
    <>
      <ActiveResources />
      <Page
        title="Dashboard"
        icon={<Box className="w-8 h-8" />}
        actions={
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              onClick={() =>
                updatePreference(
                  "showServerStats",
                  !preferences.showServerStats,
                )
              }
              className="flex items-center gap-2"
            >
              {preferences.showServerStats ? (
                <>
                  <EyeOff className="w-4 h-4" />
                  <span>Hide Server Stats</span>
                </>
              ) : (
                <>
                  <Eye className="w-4 h-4" />
                  <span>Show Server Stats</span>
                </>
              )}
            </Button>
            <ExportButton />
          </div>
        }
      >
        <div className="flex flex-col gap-6 w-full">
          {noResources && (
            <div className="flex items-center gap-4 px-2 text-muted-foreground">
              <AlertTriangle className="w-4 h-4" />
              <p className="text-lg">
                No resources found.{" "}
                {user.admin
                  ? "To get started, create a server."
                  : "Contact an admin for access to resources."}
              </p>
            </div>
          )}
          <ResourceRow type="Server" />
          <ResourceRow type="Stack" />
          <ResourceRow type="Deployment" />
          <ResourceRow type="Build" />
          <ResourceRow type="Repo" />
          <ResourceRow type="Procedure" />
          <ResourceRow type="Action" />
          <ResourceRow type="ResourceSync" />
        </div>
      </Page>
    </>
  );
}

const ResourceRow = ({ type }: { type: UsableResource }) => {
  const _recents = useUser().data?.recents?.[type]?.slice(0, 6);
  const _resources = useRead(`List${type}s`, {}).data;
  const recents = _recents?.filter(
    (recent) => !_resources?.every((resource) => resource.id !== recent),
  );
  const resources = _resources
    ?.filter((r) => !recents?.includes(r.id))
    .map((r) => r.id);
  const ids = [
    ...(recents ?? []),
    ...(resources?.slice(0, 6 - (recents?.length || 0)) ?? []),
  ];
  if (ids.length === 0) return;
  const Components = ResourceComponents[type];
  const name = type === "ResourceSync" ? "Resource Sync" : type;
  return (
    <div className="border rounded-md flex flex-col md:flex-row">
      <Link
        to={`/${usableResourcePath(type)}`}
        className="shrink-0 px-6 py-4 flex flex-col justify-center lg:border-r group bg-accent/50 hover:bg-accent/15 transition-colors"
      >
        <div className="flex items-center gap-4 text-xl group-hover:underline">
          <Components.Icon />
          {name}s
        </div>
        <Components.Dashboard />
      </Link>
      <div className="px-6 py-4 w-full flex flex-col gap-4">
        <p className="text-sm text-muted-foreground flex items-center gap-2">
          <History className="w-3" />
          Recently Viewed
        </p>
        <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-2 2xl:grid-cols-3 gap-4 auto-rows-max">
          {ids.map((id, i) => (
            <RecentCard
              key={type + id}
              type={type}
              id={id}
              className={
                i > 3
                  ? "hidden 2xl:flex"
                  : i > 1
                    ? "hidden sm:flex md:hidden xl:flex"
                    : undefined
              }
            />
          ))}
        </div>
      </div>
    </div>
  );
};

const RecentCard = ({
  type,
  id,
  className,
}: {
  type: UsableResource;
  id: string;
  className?: string;
}) => {
  const Components = ResourceComponents[type];
  const resource = Components.list_item(id);
  const { preferences } = useDashboardPreferences();

  if (!resource) return null;

  const tags = resource?.tags;
  const showServerStats = type === "Server" && preferences.showServerStats;

  return (
    <Link
      to={`${usableResourcePath(type)}/${id}`}
      className={cn(
        "w-full px-3 py-2 border rounded-md hover:bg-accent/25 hover:-translate-y-1 transition-all duration-1000 linear flex flex-col justify-between",
        showServerStats ? "min-h-32" : "h-20",
        className,
      )}
    >
      <div className="flex items-center justify-between">
        <div className="flex flex-wrap items-center gap-2 text-sm text-nowrap">
          <Components.Icon id={id} />
          <ResourceNameSimple type={type} id={id} />
          {resource.template && <TemplateMarker type={type} />}
        </div>
        {type === "Deployment" && <DeploymentUpdateAvailable id={id} small />}
        {type === "Stack" && <StackUpdateAvailable id={id} small />}
      </div>

      <div 
        className={cn(
          "overflow-hidden w-full transition-opacity transition-all duration-1000 linear",
          showServerStats 
            ? "max-h-40 opacity-100 py-2" 
            : "max-h-0 opacity-0 py-0"
        )}
      >
        <div className="flex flex-col gap-2">
          <ServerStatsMini id={id} enabled={showServerStats} />
        </div>
      </div>

      <div className="flex flex-row gap-2 w-full py-1">
        <TagsWithBadge tag_ids={tags} />
      </div>
    </Link>
  );
};

export type DashboardPieChartItem = {
  title: string;
  intention: ColorIntention;
  value: number;
};

export const DashboardPieChart = ({
  data: _data,
}: {
  data: Array<DashboardPieChartItem | false | undefined>;
}) => {
  const data = _data.filter((d) => d) as Array<DashboardPieChartItem>;
  return (
    <div className="flex items-center gap-8">
      <div className="flex flex-col gap-2 w-28">
        {data.map(({ title, value, intention }) => (
          <p key={title} className="flex gap-2 text-xs text-muted-foreground">
            <span
              className={cn(
                "font-bold",
                text_color_class_by_intention(intention)
              )}
            >
              {value}
            </span>
            {title}
          </p>
        ))}
      </div>
      <PieChart
        className="w-32 h-32"
        radius={42}
        lineWidth={30}
        data={data.map(({ title, value, intention }) => ({
          title,
          value,
          color: hex_color_by_intention(intention),
        }))}
      />
    </div>
  );
};

const ActiveResources = () => {
  const builds =
    useRead("ListBuilds", {}).data?.filter(
      (build) => build.info.state === Types.BuildState.Building
    ) ?? [];
  const repos =
    useRead("ListRepos", {}).data?.filter((repo) =>
      [
        Types.RepoState.Building,
        Types.RepoState.Cloning,
        Types.RepoState.Pulling,
      ].includes(repo.info.state)
    ) ?? [];
  const procedures =
    useRead("ListProcedures", {}).data?.filter(
      (procedure) => procedure.info.state === Types.ProcedureState.Running
    ) ?? [];
  const actions =
    useRead("ListActions", {}).data?.filter(
      (action) => action.info.state === Types.ActionState.Running
    ) ?? [];

  const resources = [
    ...(builds ?? []).map((build) => ({
      type: "Build" as UsableResource,
      id: build.id,
      state: (
        <StatusBadge
          text={build.info.state}
          intent={build_state_intention(build.info.state)}
        />
      ),
    })),
    ...(repos ?? []).map((repo) => ({
      type: "Repo" as UsableResource,
      id: repo.id,
      state: (
        <StatusBadge
          text={repo.info.state}
          intent={repo_state_intention(repo.info.state)}
        />
      ),
    })),
    ...(procedures ?? []).map((procedure) => ({
      type: "Procedure" as UsableResource,
      id: procedure.id,
      state: (
        <StatusBadge
          text={procedure.info.state}
          intent={procedure_state_intention(procedure.info.state)}
        />
      ),
    })),
    ...(actions ?? []).map((action) => ({
      type: "Action" as UsableResource,
      id: action.id,
      state: (
        <StatusBadge
          text={action.info.state}
          intent={action_state_intention(action.info.state)}
        />
      ),
    })),
  ];

  if (resources.length === 0) return null;

  return (
    <div className="mb-12">
      <Section
        title="Active"
        icon={
          <Circle className="w-4 h-4 stroke-none transition-colors fill-green-500" />
        }
      >
        <DataTable
          tableKey="active-resources"
          data={resources}
          columns={[
            {
              accessorKey: "name",
              header: ({ column }) => (
                <SortableHeader column={column} title="Name" />
              ),
              cell: ({ row }) => (
                <ResourceLink type={row.original.type} id={row.original.id} />
              ),
            },
            {
              accessorKey: "type",
              header: ({ column }) => (
                <SortableHeader column={column} title="Resource" />
              ),
            },
            {
              header: "State",
              cell: ({ row }) => row.original.state,
            },
          ]}
        />
      </Section>
    </div>
  );
};
