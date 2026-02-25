import { useInvalidate, useUserTargetPermissions, useWrite } from "@/lib/hooks";
import { levelSortingFn } from "@/lib/utils";
import { ResourceComponents, UsableResource } from "@/resources";
import ResourceLink from "@/resources/link";
import { DataTable, SortableHeader } from "@/ui/data-table";
import SearchInput from "@/ui/search-input";
import Section, { SectionProps } from "@/ui/section";
import { Group, Text } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { Types } from "komodo_client";
import { useState } from "react";
import ResourceTypeSelector from "../resource-type-selector";
import LabelledSwitch from "@/ui/labelled-switch";
import PermissionLevelSelector from "./level-selector";
import SpecificPermissionSelector from "./specific-selector";

export interface SpecificPermissionsTableProps extends SectionProps {
  userTarget: Types.UserTarget;
}

export default function SpecificPermissionsSection({
  userTarget,
  ...sectionProps
}: SpecificPermissionsTableProps) {
  const [showAll, setShowAll] = useState(false);
  const [resourceType, setResourceType] = useState<UsableResource | null>(null);
  const permissions = useUserTargetPermissions(userTarget);
  const inv = useInvalidate();
  const { mutate: update } = useWrite("UpdatePermissionOnTarget", {
    onSuccess: () => {
      inv(["ListUserTargetPermissions"]);
      notifications.show({ message: "Updated permission", color: "green" });
    },
  });
  const [search, setSearch] = useState("");
  const searchSplit = search.toLowerCase().split(" ");
  const tableData =
    permissions?.filter(
      (permission) =>
        (resourceType === null
          ? true
          : permission.resource_target.type === resourceType) &&
        (showAll ? true : permission.level !== Types.PermissionLevel.None) &&
        searchSplit.every(
          (search) =>
            permission.name.toLowerCase().includes(search) ||
            permission.resource_target.type.toLowerCase().includes(search),
        ),
    ) ?? [];
  return (
    <Section
      title="Per Resource Permissions"
      titleFz="h3"
      titleMb="0"
      actions={
        <Group>
          <SearchInput value={search} onSearch={setSearch} />
          <ResourceTypeSelector
            value={resourceType}
            onChange={setResourceType}
          />
          <LabelledSwitch
            checked={showAll}
            onCheckedChange={setShowAll}
            label="Show All"
          />
        </Group>
      }
      {...sectionProps}
    >
      <DataTable
        tableKey="specific-permissions-v1"
        data={tableData}
        columns={[
          {
            accessorKey: "resource_target.type",
            size: 150,
            header: ({ column }) => (
              <SortableHeader column={column} title="Resource" />
            ),
            cell: ({ row }) => {
              const RC =
                ResourceComponents[
                  row.original.resource_target.type as UsableResource
                ];
              return (
                <Group gap="sm">
                  <RC.Icon />
                  <Text>{row.original.resource_target.type}</Text>
                </Group>
              );
            },
          },
          {
            accessorKey: "resource_target",
            size: 250,
            header: ({ column }) => (
              <SortableHeader column={column} title="Target" />
            ),
            cell: ({
              row: {
                original: { resource_target },
              },
            }) => {
              return (
                <ResourceLink
                  type={resource_target.type as UsableResource}
                  id={resource_target.id}
                />
              );
            },
          },
          {
            accessorKey: "level",
            size: 150,
            sortingFn: (a, b) =>
              levelSortingFn(a.original.level, b.original.level),
            header: ({ column }) => (
              <SortableHeader column={column} title="Level" />
            ),
            cell: ({ row: { original: permission } }) => (
              <PermissionLevelSelector
                level={permission.level ?? Types.PermissionLevel.None}
                onChange={(value) =>
                  update({
                    ...permission,
                    user_target: userTarget,
                    permission: {
                      level: value,
                      specific: permission.specific ?? [],
                    },
                  })
                }
              />
            ),
          },
          {
            header: "Specific",
            size: 300,
            cell: ({ row: { original: permission } }) => {
              return (
                <SpecificPermissionSelector
                  type={permission.resource_target.type as UsableResource}
                  specific={permission.specific ?? []}
                  onChange={(specific) => {
                    update({
                      ...permission,
                      user_target: userTarget,
                      permission: {
                        level: permission.level ?? Types.PermissionLevel.None,
                        specific,
                      },
                    });
                  }}
                />
              );
            },
          },
        ]}
      />
    </Section>
  );
}
