import { useAllResources } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import { UsableResource } from "@/resources";
import ResourceLink from "@/resources/link";
import { ICONS } from "@/theme/icons";
import { ConfigItem } from "@/ui/config/item";
import { DataTable, SortableHeader } from "@/ui/data-table";
import SearchInput from "@/ui/search-input";
import { Button, Group, Modal, Stack, Switch, Text } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { Types } from "komodo_client";
import { useState } from "react";

export default function AlerterConfigResources({
  resources,
  set,
  disabled,
  blacklist,
}: {
  resources: Types.ResourceTarget[];
  set: (resources: Types.ResourceTarget[]) => void;
  disabled: boolean;
  blacklist: boolean;
}) {
  const resourcesMap = useAllResources();
  const allResources = [
    ...(resourcesMap.Server?.map((server) => {
      return {
        type: "Server",
        id: server.id,
        name: server.name.toLowerCase(),
        enabled: resources.find(
          (r) => r.type === "Server" && r.id === server.id,
        )
          ? true
          : false,
      };
    }) ?? []),
    ...(resourcesMap.Swarm?.map((swarm) => {
      return {
        type: "Swarm",
        id: swarm.id,
        name: swarm.name.toLowerCase(),
        enabled: resources.find((r) => r.type === "Swarm" && r.id === swarm.id)
          ? true
          : false,
      };
    }) ?? []),
    ...(resourcesMap.Stack?.map((stack) => {
      return {
        type: "Stack",
        id: stack.id,
        name: stack.name.toLowerCase(),
        enabled: resources.find((r) => r.type === "Stack" && r.id === stack.id)
          ? true
          : false,
      };
    }) ?? []),
    ...(resourcesMap.Deployment?.map((deployment) => ({
      type: "Deployment",
      id: deployment.id,
      name: deployment.name.toLowerCase(),
      enabled: resources.find(
        (r) => r.type === "Deployment" && r.id === deployment.id,
      )
        ? true
        : false,
    })) ?? []),
    ...(resourcesMap.Build?.map((build) => ({
      type: "Build",
      id: build.id,
      name: build.name.toLowerCase(),
      enabled: resources.find((r) => r.type === "Build" && r.id === build.id)
        ? true
        : false,
    })) ?? []),
    ...(resourcesMap.Repo?.map((repo) => ({
      type: "Repo",
      id: repo.id,
      name: repo.name.toLowerCase(),
      enabled: resources.find((r) => r.type === "Repo" && r.id === repo.id)
        ? true
        : false,
    })) ?? []),
    ...(resourcesMap.ResourceSync?.map((sync) => ({
      type: "ResourceSync",
      id: sync.id,
      name: sync.name.toLowerCase(),
      enabled: resources.find(
        (r) => r.type === "ResourceSync" && r.id === sync.id,
      )
        ? true
        : false,
    })) ?? []),
  ];

  const [opened, { open, close }] = useDisclosure();
  const [search, setSearch] = useState("");
  const filtered = filterBySplit(
    allResources,
    search,
    (resource) => resource.name,
  );

  return (
    <>
      <ConfigItem label={`Resource ${blacklist ? "Blacklist" : "Whitelist"}`}>
        <Group>
          <Button leftSection={<ICONS.Edit size="1rem" />} onClick={open}>
            Edit Resources
          </Button>
          {resources.length ? (
            <Text c="dimmed">
              Alerts {blacklist ? "blacklisted" : "whitelisted"} by{" "}
              <b>{resources.length}</b> resources
            </Text>
          ) : undefined}
        </Group>
      </ConfigItem>
      <Modal
        opened={opened}
        onClose={close}
        title={`Edit ${blacklist ? "Blacklisted" : "Whitelisted"} Resources`}
        size="xl"
      >
        <Stack gap="xs">
          <SearchInput value={search} onSearch={setSearch} />
          <DataTable
            tableKey="alerter-resources"
            data={filtered}
            columns={[
              {
                accessorKey: "type",
                header: ({ column }) => (
                  <SortableHeader column={column} title="Resource" />
                ),
                cell: ({ row }) => {
                  const Icon = ICONS[row.original.type as UsableResource];
                  return (
                    <Group gap="xs">
                      <Icon size="1rem" />
                      {row.original.type}
                    </Group>
                  );
                },
              },
              {
                accessorKey: "id",
                header: ({ column }) => (
                  <SortableHeader column={column} title="Target" />
                ),
                cell: ({ row: { original: resource_target } }) => {
                  return (
                    <ResourceLink
                      type={resource_target.type as UsableResource}
                      id={resource_target.id}
                    />
                  );
                },
              },
              {
                accessorKey: "enabled",
                header: ({ column }) => (
                  <SortableHeader
                    column={column}
                    title={blacklist ? "Blacklist" : "Whitelist"}
                  />
                ),
                cell: ({ row }) => {
                  return (
                    <Switch
                      disabled={disabled}
                      checked={row.original.enabled}
                      onChange={() => {
                        if (row.original.enabled) {
                          set(
                            resources.filter(
                              (r) =>
                                r.type !== row.original.type ||
                                r.id !== row.original.id,
                            ),
                          );
                        } else {
                          set([
                            ...resources,
                            {
                              type: row.original.type as UsableResource,
                              id: row.original.id,
                            },
                          ]);
                        }
                      }}
                    />
                  );
                },
              },
            ]}
          />
        </Stack>
      </Modal>
    </>
  );
}
