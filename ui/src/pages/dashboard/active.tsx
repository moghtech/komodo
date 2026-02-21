import {
  actionStateIntention,
  buildStateIntention,
  hexColorByIntention,
  procedureStateIntention,
  repoStateIntention,
} from "@/lib/color";
import { useRead } from "@/lib/hooks";
import ResourceLink from "@/resources/link";
import { ICONS } from "@/theme/icons";
import { DataTable, SortableHeader } from "@/ui/data-table";
import Section from "@/ui/section";
import StatusBadge from "@/ui/status-badge";
import { Group } from "@mantine/core";
import { Types } from "komodo_client";
import { Circle } from "lucide-react";
import { ReactNode } from "react";

export default function DashboardActiveResources() {
  const builds =
    useRead("ListBuilds", {}).data?.filter(
      (build) => build.info.state === Types.BuildState.Building,
    ) ?? [];
  const repos =
    useRead("ListRepos", {}).data?.filter((repo) =>
      [
        Types.RepoState.Building,
        Types.RepoState.Cloning,
        Types.RepoState.Pulling,
      ].includes(repo.info.state),
    ) ?? [];
  const procedures =
    useRead("ListProcedures", {}).data?.filter(
      (procedure) => procedure.info.state === Types.ProcedureState.Running,
    ) ?? [];
  const actions =
    useRead("ListActions", {}).data?.filter(
      (action) => action.info.state === Types.ActionState.Running,
    ) ?? [];
  const globalAutoUpdates =
    useRead("ListUpdates", {
      query: {
        operation: Types.Operation.GlobalAutoUpdate,
        status: Types.UpdateStatus.InProgress,
      },
    }).data?.updates ?? [];

  const resources: {
    type: Types.ResourceTarget["type"];
    id: string;
    label?: string;
    state: ReactNode;
  }[] = [
    ...builds.map((build) => ({
      type: "Build" as Types.ResourceTarget["type"],
      id: build.id,
      state: (
        <StatusBadge
          text={build.info.state}
          intent={buildStateIntention(build.info.state)}
        />
      ),
    })),
    ...repos.map((repo) => ({
      type: "Repo" as Types.ResourceTarget["type"],
      id: repo.id,
      state: (
        <StatusBadge
          text={repo.info.state}
          intent={repoStateIntention(repo.info.state)}
        />
      ),
    })),
    ...procedures.map((procedure) => ({
      type: "Procedure" as Types.ResourceTarget["type"],
      id: procedure.id,
      state: (
        <StatusBadge
          text={procedure.info.state}
          intent={procedureStateIntention(procedure.info.state)}
        />
      ),
    })),
    ...actions.map((action) => ({
      type: "Action" as Types.ResourceTarget["type"],
      id: action.id,
      state: (
        <StatusBadge
          text={action.info.state}
          intent={actionStateIntention(action.info.state)}
        />
      ),
    })),
    ...globalAutoUpdates.map((update) => ({
      type: "System" as Types.ResourceTarget["type"],
      id: update.id,
      label: "Global Auto Update",
      state: (
        <StatusBadge
          text={Types.ActionState.Running}
          intent={actionStateIntention(Types.ActionState.Running)}
        />
      ),
    })),
  ];

  if (resources.length === 0) return null;

  return (
    <Section
      title="Active"
      icon={
        <Circle
          size="1rem"
          stroke={hexColorByIntention("Good")}
          fill={hexColorByIntention("Good")}
        />
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
            cell: ({ row }) =>
              row.original.type === "System" ? (
                <Group gap="xs">
                  <ICONS.System size="1rem" />
                  {row.original.label ?? "System"}
                </Group>
              ) : (
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
  );
}
