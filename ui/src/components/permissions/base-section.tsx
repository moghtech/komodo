import { useInvalidate, useRead, useWrite } from "@/lib/hooks";
import { filterBySplit, levelSortingFn } from "@/lib/utils";
import {
  RESOURCE_TARGETS,
  ResourceComponents,
  UsableResource,
} from "@/resources";
import { DataTable, SortableHeader } from "mogh_ui";
import { Section, SectionProps } from "mogh_ui";
import { Group, Text } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { Types } from "komodo_client";
import { useState } from "react";
import PermissionLevelSelector from "./level-selector";
import SpecificPermissionSelector from "./specific-selector";
import { SearchInput } from "mogh_ui";
import { LabelledSwitch } from "mogh_ui";

export interface BasePermissionsSectionProps extends SectionProps {
  userTarget: Types.UserTarget;
}

export default function BasePermissionsSection({
  userTarget,
  ...sectionProps
}: BasePermissionsSectionProps) {
  const [showAll, setShowAll] = useState(false);
  const [search, setSearch] = useState("");

  const { data: user, isPending: userPending } = useRead(
    "FindUser",
    { user: userTarget.id },
    { enabled: userTarget.type === "User" },
  );
  const { data: group, isPending: groupPending } = useRead(
    "GetUserGroup",
    { user_group: userTarget.id },
    { enabled: userTarget.type === "UserGroup" },
  );

  const inv = useInvalidate();
  const { mutate: update } = useWrite("UpdatePermissionOnResourceType", {
    onSuccess: () => {
      notifications.show({
        message: "Updated permissions on target.",
        color: "green",
      });
      if (userTarget.type === "User") {
        inv(["FindUser", { user: userTarget.id }]);
      } else if (userTarget.type === "UserGroup") {
        inv(["GetUserGroup", { user_group: userTarget.id }]);
      }
    },
  });

  const all = userTarget.type === "User" ? user?.all : group?.all;
  const permissions = RESOURCE_TARGETS.map((type) => {
    const permission = all?.[type] ?? Types.PermissionLevel.None;
    return {
      type,
      level: typeof permission === "string" ? permission : permission.level,
      specific: typeof permission === "string" ? [] : permission.specific,
    };
  }).filter(
    (item) =>
      showAll ||
      item.level !== Types.PermissionLevel.None ||
      item.specific.length !== 0,
  );

  const filtered = filterBySplit(permissions, search, (p) => p.type);

  return (
    <Section
      isPending={userTarget.type === "User" ? userPending : groupPending}
      error={
        userTarget.type === "User"
          ? !user
            ? "No user matching ID"
            : undefined
          : !group
            ? "No group matching ID"
            : undefined
      }
      title="Base Permissions on Resource Types"
      titleFz="h3"
     
      actions={
        <Group>
          <SearchInput value={search} onSearch={setSearch} />
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
        tableKey="base-permissions-v1"
        data={filtered}
        columns={[
          {
            accessorKey: "type",
            size: 150,
            header: ({ column }) => (
              <SortableHeader column={column} title="Resource Type" />
            ),
            cell: ({ row }) => {
              const RC =
                ResourceComponents[row.original.type as UsableResource];
              return (
                <Group gap="sm">
                  <RC.Icon />
                  <Text>{row.original.type}</Text>
                </Group>
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
            cell: ({ row }) => (
              <PermissionLevelSelector
                level={row.original.level ?? Types.PermissionLevel.None}
                onChange={(level) => {
                  update({
                    user_target: userTarget,
                    resource_type: row.original.type,
                    permission: {
                      level,
                      specific: row.original.specific,
                    },
                  });
                }}
              />
            ),
          },
          {
            header: "Specific",
            size: 300,
            cell: ({ row }) => {
              return (
                <SpecificPermissionSelector
                  type={row.original.type}
                  specific={row.original.specific}
                  onChange={(specific) => {
                    update({
                      user_target: userTarget,
                      resource_type: row.original.type,
                      permission: {
                        level: row.original.level,
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
