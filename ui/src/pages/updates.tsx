import ResourceTypeSelector from "@/components/resource-type-selector";
import { useUpdateDetails } from "@/components/updates/details";
import UserAvatar from "@/components/user-avatar";
import {
  fmtDateWithMinutes,
  fmtOperation,
  fmtUpperCamelcase,
} from "@/lib/formatting";
import { useRead, useSetTitle } from "@/lib/hooks";
import { UsableResource } from "@/resources";
import ResourceLink from "@/resources/link";
import ResourceSelector from "@/resources/selector";
import { ICONS } from "@/theme/icons";
import { DataTable } from "@/ui/data-table";
import Page from "@/ui/page";
import StatusBadge from "@/ui/status-badge";
import {
  ActionIcon,
  Group,
  Pagination,
  Select,
  Stack,
  Text,
} from "@mantine/core";
import { Types } from "komodo_client";
import { useState } from "react";
import { useSearchParams } from "react-router-dom";

export default function Updates() {
  useSetTitle("Updates");
  const [page, setPage] = useState(0);
  const [params, setParams] = useSearchParams();
  const { open: openDetails } = useUpdateDetails();

  const { type, id, operation } = {
    type: params.get("type") as UsableResource | null,
    id: params.get("id"),
    operation: params.get("operation") as Types.Operation | null,
  };

  const { data: updates } = useRead("ListUpdates", {
    query: {
      "target.type": type ?? undefined,
      "target.id": id ?? undefined,
      operation: operation ?? undefined,
    },
    page,
  });

  return (
    <Page
      title="Updates"
      icon={ICONS.Update}
      description="View historical updates"
    >
      <Stack>
        {/* QUERY */}
        <Group>
          {/* RESOURCE TYPE */}
          <ResourceTypeSelector
            value={type}
            onChange={(type) => {
              const p = new URLSearchParams(params.toString());
              if (type) {
                p.set("type", type);
              } else {
                p.delete("type");
              }
              p.delete("id");
              p.delete("operation");
              setParams(p);
            }}
          />

          {/* RESOURCE ID */}
          {type && (
            <ResourceSelector
              type={type}
              selected={id ?? undefined}
              onSelect={(id) => {
                const p = new URLSearchParams(params.toString());
                id === "" ? p.delete("id") : p.set("id", id);
                setParams(p);
              }}
              targetProps={{ w: { base: "100%", xs: 250 } }}
            />
          )}

          {/* OPERATION */}
          <Select
            value={operation}
            placeholder="Select operation"
            onChange={(op) => {
              const p = new URLSearchParams(params.toString());
              if (op) {
                p.set("operation", op);
              } else {
                p.delete("operation");
              }
              setParams(p);
            }}
            data={(type
              ? OPERATIONS_BY_RESOURCE[type]
              : Object.values(Types.Operation).filter(
                  (o) => o !== Types.Operation.None,
                )
            ).map((value) => ({ value, label: fmtUpperCamelcase(value) }))}
            w={{ base: "100%", xs: 250 }}
            searchable
            clearable
          />

          {/* RESET */}
          <ActionIcon
            onClick={() => setParams({})}
            variant="filled"
            color="red"
            disabled={!params.size}
          >
            <ICONS.Clear size="1rem" />
          </ActionIcon>

          {/* PAGINATION */}
          <Pagination.Root
            total={updates?.next_page ? page + 2 : page + 1}
            value={page + 1}
            onChange={(page) => setPage(page - 1)}
          >
            <Group gap="0.2rem" justify="center">
              <Pagination.First />
              <Pagination.Previous />
              <Pagination.Items />
              <Pagination.Next />
            </Group>
          </Pagination.Root>
        </Group>

        <DataTable
          tableKey="updates"
          data={updates?.updates ?? []}
          columns={[
            {
              header: "Details",
              cell: () => {
                return (
                  <ActionIcon>
                    <ICONS.Search size="1rem" />
                  </ActionIcon>
                );
              },
            },
            !params.get("id") && {
              header: "Resource",
              cell: ({ row }) =>
                row.original.target.type === "System" ? (
                  <Group gap="xs">
                    <ICONS.System size="1rem" />
                    <Text>System</Text>
                  </Group>
                ) : (
                  <ResourceLink
                    type={row.original.target.type}
                    id={row.original.target.id}
                  />
                ),
            },
            {
              header: "Operation",
              accessorKey: "operation",
              cell: ({ row }) => {
                const more =
                  row.original.status === Types.UpdateStatus.InProgress
                    ? "in progress"
                    : row.original.status === Types.UpdateStatus.Queued
                      ? "queued"
                      : undefined;
                return (
                  <Group gap="xs">
                    <Text>{fmtOperation(row.original.operation)}</Text>
                    {more && <Text c="dimmed">{more}</Text>}
                  </Group>
                );
              },
            },
            {
              header: "Result",
              cell: ({ row }) => {
                const { success, status } = row.original;
                return (
                  <StatusBadge
                    intent={
                      status === Types.UpdateStatus.Complete
                        ? success
                          ? "Good"
                          : "Critical"
                        : "None"
                    }
                    text={
                      status === Types.UpdateStatus.Complete
                        ? success
                          ? "Success"
                          : "Failed"
                        : "Processing"
                    }
                  />
                );
              },
            },
            {
              header: "Start Time",
              accessorFn: ({ start_ts }) =>
                fmtDateWithMinutes(new Date(start_ts)),
            },
            {
              header: "Operator",
              accessorKey: "operator",
              cell: ({ row }) => (
                <UserAvatar userId={row.original.operator} fz="md" />
              ),
            },
          ]}
          onRowClick={(row) => openDetails(row.id)}
        />
      </Stack>
    </Page>
  );
}

const OPERATIONS_BY_RESOURCE: { [key: string]: Types.Operation[] } = {
  Server: [
    Types.Operation.CreateServer,
    Types.Operation.UpdateServer,
    Types.Operation.DeleteServer,
    Types.Operation.RenameServer,
    Types.Operation.StartContainer,
    Types.Operation.RestartContainer,
    Types.Operation.PauseContainer,
    Types.Operation.UnpauseContainer,
    Types.Operation.StopContainer,
    Types.Operation.DestroyContainer,
    Types.Operation.StartAllContainers,
    Types.Operation.RestartAllContainers,
    Types.Operation.PauseAllContainers,
    Types.Operation.UnpauseAllContainers,
    Types.Operation.StopAllContainers,
    Types.Operation.PruneContainers,
    Types.Operation.CreateNetwork,
    Types.Operation.DeleteNetwork,
    Types.Operation.PruneNetworks,
    Types.Operation.DeleteImage,
    Types.Operation.PruneImages,
    Types.Operation.DeleteVolume,
    Types.Operation.PruneVolumes,
    Types.Operation.PruneDockerBuilders,
    Types.Operation.PruneBuildx,
    Types.Operation.PruneSystem,
  ],
  Stack: [
    Types.Operation.CreateStack,
    Types.Operation.UpdateStack,
    Types.Operation.RenameStack,
    Types.Operation.DeleteStack,
    Types.Operation.WriteStackContents,
    Types.Operation.RefreshStackCache,
    Types.Operation.DeployStack,
    Types.Operation.StartStack,
    Types.Operation.RestartStack,
    Types.Operation.PauseStack,
    Types.Operation.UnpauseStack,
    Types.Operation.StopStack,
    Types.Operation.DestroyStack,
    Types.Operation.StartStackService,
    Types.Operation.RestartStackService,
    Types.Operation.PauseStackService,
    Types.Operation.UnpauseStackService,
    Types.Operation.StopStackService,
  ],
  Deployment: [
    Types.Operation.CreateDeployment,
    Types.Operation.UpdateDeployment,
    Types.Operation.DeleteDeployment,
    Types.Operation.Deploy,
    Types.Operation.StartDeployment,
    Types.Operation.RestartDeployment,
    Types.Operation.PauseDeployment,
    Types.Operation.UnpauseDeployment,
    Types.Operation.StopDeployment,
    Types.Operation.DestroyDeployment,
    Types.Operation.RenameDeployment,
  ],
  Build: [
    Types.Operation.CreateBuild,
    Types.Operation.UpdateBuild,
    Types.Operation.DeleteBuild,
    Types.Operation.RunBuild,
    Types.Operation.CancelBuild,
  ],
  Repo: [
    Types.Operation.CreateRepo,
    Types.Operation.UpdateRepo,
    Types.Operation.DeleteRepo,
    Types.Operation.CloneRepo,
    Types.Operation.PullRepo,
    Types.Operation.BuildRepo,
    Types.Operation.CancelRepoBuild,
  ],
  Procedure: [
    Types.Operation.CreateProcedure,
    Types.Operation.UpdateProcedure,
    Types.Operation.DeleteProcedure,
    Types.Operation.RunProcedure,
  ],
  Builder: [
    Types.Operation.CreateBuilder,
    Types.Operation.UpdateBuilder,
    Types.Operation.DeleteBuilder,
  ],
  Alerter: [
    Types.Operation.CreateAlerter,
    Types.Operation.UpdateAlerter,
    Types.Operation.DeleteAlerter,
  ],
  ResourceSync: [
    Types.Operation.CreateResourceSync,
    Types.Operation.UpdateResourceSync,
    Types.Operation.DeleteResourceSync,
    Types.Operation.CommitSync,
    Types.Operation.RunSync,
  ],
};
