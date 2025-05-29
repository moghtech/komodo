import { useInvalidate, useLocalStorage, useRead, useWrite } from "@lib/hooks";
import { Types } from "komodo_client";
import { UsableResource } from "@types";
import { useToast } from "@ui/use-toast";
import { ReactNode, useState } from "react";
import { useUserTargetPermissions } from "./hooks";
import { Section } from "@components/layouts";
import { Input } from "@ui/input";
import { ResourceComponents } from "@components/resources";
import { Label } from "@ui/label";
import { Switch } from "@ui/switch";
import { DataTable, SortableHeader } from "@ui/data-table";
import { level_to_number, resource_name, RESOURCE_TARGETS } from "@lib/utils";
import { ResourceLink } from "@components/resources/common";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import {
  PermissionLevelSelector,
  SpecificPermissionSelector,
} from "@components/config/util";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@ui/tabs";

export const PermissionsTableTabs = ({
  user_target,
}: {
  user_target: Types.UserTarget;
}) => {
  const [view, setView] = useLocalStorage<"Base" | "Specific">(
    "user-permissions-tab-v1",
    "Base"
  );
  const tabs = (
    <TabsList>
      <TabsTrigger value="Base" className="w-[100px]">
        Base
      </TabsTrigger>
      <TabsTrigger value="Specific" className="w-[100px]">
        Specific
      </TabsTrigger>
    </TabsList>
  );
  return (
    <Tabs value={view} onValueChange={setView as any}>
      <TabsContent value="Base">
        <BasePermissionsTable user_target={user_target} titleOther={tabs} />
      </TabsContent>
      <TabsContent value="Specific">
        <SpecificPermissionsTable user_target={user_target} titleOther={tabs} />
      </TabsContent>
    </Tabs>
  );
};

export const SpecificPermissionsTable = ({
  user_target,
  titleOther,
}: {
  user_target: Types.UserTarget;
  titleOther: ReactNode;
}) => {
  const { toast } = useToast();
  const [showNone, setShowNone] = useState(false);
  const [resourceType, setResourceType] = useState<UsableResource | "All">(
    "All"
  );
  const [search, setSearch] = useState("");
  const searchSplit = search.toLowerCase().split(" ");
  const inv = useInvalidate();
  const permissions = useUserTargetPermissions(user_target);
  const { mutate } = useWrite("UpdatePermissionOnTarget", {
    onSuccess: () => {
      toast({ title: "Updated permission" });
      inv(["ListUserTargetPermissions"]);
    },
  });
  return (
    <Section
      titleOther={titleOther}
      actions={
        <div className="flex gap-6 items-center">
          <Input
            placeholder="search"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-[300px]"
          />
          <Select
            value={resourceType}
            onValueChange={(value) =>
              setResourceType(value as UsableResource | "All")
            }
          >
            <SelectTrigger className="w-44">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {["All", ...Object.keys(ResourceComponents)].map((type) => (
                <SelectItem key={type} value={type}>
                  {type === "All" ? "All" : type + "s"}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <div
            className="flex gap-3 items-center"
            onClick={() => setShowNone(!showNone)}
          >
            <Label htmlFor="show-none">Show All Resources</Label>
            <Switch id="show-none" checked={showNone} />
          </div>
        </div>
      }
    >
      <DataTable
        tableKey="permissions"
        data={
          permissions?.filter(
            (permission) =>
              (resourceType === "All"
                ? true
                : permission.resource_target.type === resourceType) &&
              (showNone
                ? true
                : permission.level !== Types.PermissionLevel.None) &&
              searchSplit.every(
                (search) =>
                  permission.name.toLowerCase().includes(search) ||
                  permission.resource_target.type.toLowerCase().includes(search)
              )
          ) ?? []
        }
        columns={[
          {
            accessorKey: "resource_target.type",
            header: ({ column }) => (
              <SortableHeader column={column} title="Resource" />
            ),
            cell: ({ row }) => {
              const Components =
                ResourceComponents[
                  row.original.resource_target.type as UsableResource
                ];
              return (
                <div className="flex gap-2 items-center">
                  <Components.Icon />
                  {row.original.resource_target.type}
                </div>
              );
            },
          },
          {
            accessorKey: "resource_target",
            sortingFn: (a, b) => {
              const ra = resource_name(
                a.original.resource_target.type as UsableResource,
                a.original.resource_target.id
              );
              const rb = resource_name(
                b.original.resource_target.type as UsableResource,
                b.original.resource_target.id
              );

              if (!ra && !rb) return 0;
              if (!ra) return -1;
              if (!rb) return 1;

              if (ra > rb) return 1;
              else if (ra < rb) return -1;
              else return 0;
            },
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
            sortingFn: (a, b) => {
              const al = level_to_number(a.original.level);
              const bl = level_to_number(b.original.level);
              const dif = al - bl;
              return dif === 0 ? 0 : dif / Math.abs(dif);
            },
            header: ({ column }) => (
              <SortableHeader column={column} title="Level" />
            ),
            cell: ({ row: { original: permission } }) => (
              <PermissionLevelSelector
                level={permission.level ?? Types.PermissionLevel.None}
                onSelect={(value) =>
                  mutate({
                    ...permission,
                    user_target,
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
            cell: ({ row: { original: permission } }) => {
              return (
                <SpecificPermissionSelector
                  type={permission.resource_target.type as UsableResource}
                  specific={permission.specific ?? []}
                  onSelect={(specific_permission) => {
                    const _specific = permission.specific ?? [];
                    const specific = (
                      _specific.includes(specific_permission)
                        ? _specific.filter((p) => p !== specific_permission)
                        : [..._specific, specific_permission]
                    ).sort();
                    mutate({
                      ...permission,
                      user_target,
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
};

export const BasePermissionsTable = ({
  user_target,
  titleOther,
}: {
  user_target: Types.UserTarget;
  titleOther: ReactNode;
}) => {
  const { toast } = useToast();
  const inv = useInvalidate();

  const { mutate } = useWrite("UpdatePermissionOnResourceType", {
    onSuccess: () => {
      toast({ title: "Updated permissions on target" });
      if (user_target.type === "User") {
        inv(["FindUser", { user: user_target.id }]);
      } else if (user_target.type === "UserGroup") {
        inv(["GetUserGroup", { user_group: user_target.id }]);
      }
    },
  });

  const update = (resource_type, permission) =>
    mutate({ user_target, resource_type, permission });

  if (user_target.type === "User") {
    return (
      <UserPermissionsOnResourceType
        user_id={user_target.id}
        update={update}
        titleOther={titleOther}
      />
    );
  } else if (user_target.type === "UserGroup") {
    return (
      <UserGroupPermissionsOnResourceType
        group_id={user_target.id}
        update={update}
        titleOther={titleOther}
      />
    );
  }
};

const UserPermissionsOnResourceType = ({
  user_id,
  update,
  titleOther,
}: {
  user_id: string;
  update: (
    resource_type: Types.ResourceTarget["type"],
    permission: Types.PermissionLevel
  ) => void;
  titleOther: ReactNode;
}) => {
  const user = useRead("FindUser", { user: user_id }).data;
  return (
    <PermissionsOnResourceType
      all={user?.all}
      update={update}
      titleOther={titleOther}
    />
  );
};

const UserGroupPermissionsOnResourceType = ({
  group_id,
  update,
  titleOther,
}: {
  group_id: string;
  update: (
    resource_type: Types.ResourceTarget["type"],
    permission: Types.PermissionLevel
  ) => void;
  titleOther: ReactNode;
}) => {
  const group = useRead("GetUserGroup", { user_group: group_id }).data;
  return (
    <PermissionsOnResourceType
      all={group?.all}
      update={update}
      titleOther={titleOther}
    />
  );
};

const PermissionsOnResourceType = ({
  all,
  update,
  titleOther,
}: {
  all: Types.User["all"];
  update: (
    resource_type: Types.ResourceTarget["type"],
    permission: Types.PermissionLevel
  ) => void;
  titleOther: ReactNode;
}) => {
  const data = RESOURCE_TARGETS.map((type) => {
    const permission = all?.[type] ?? Types.PermissionLevel.None;
    return {
      type,
      level: typeof permission === "string" ? permission : permission.level,
      specific: typeof permission === "string" ? [] : permission.specific,
    };
  });
  return (
    <Section titleOther={titleOther}>
      <DataTable
        tableKey="permissions"
        data={data}
        columns={[
          {
            accessorKey: "type",
            header: ({ column }) => (
              <SortableHeader column={column} title="Resource Type" />
            ),
            cell: ({ row }) => {
              const Components =
                ResourceComponents[row.original.type as UsableResource];
              return (
                <div className="flex gap-2 items-center">
                  <Components.Icon />
                  {row.original.type}
                </div>
              );
            },
          },
          {
            accessorKey: "level",
            sortingFn: (a, b) => {
              const al = level_to_number(a.original.level);
              const bl = level_to_number(b.original.level);
              const dif = al - bl;
              return dif === 0 ? 0 : dif / Math.abs(dif);
            },
            header: ({ column }) => (
              <SortableHeader column={column} title="Level" />
            ),
            cell: ({ row }) => (
              <PermissionLevelSelector
                level={row.original.level ?? Types.PermissionLevel.None}
                onSelect={(value) => {
                  update(row.original.type, value);
                }}
              />
            ),
          },
          {
            header: "Specific",
            cell: ({ row }) => {
              return (
                <SpecificPermissionSelector
                  type={row.original.type}
                  specific={row.original.specific}
                  onSelect={(specific_permission) => {
                    const _specific = row.original.specific ?? [];
                    const specific = (
                      _specific.includes(specific_permission)
                        ? _specific.filter((p) => p !== specific_permission)
                        : [..._specific, specific_permission]
                    ).sort();
                    // mutate({
                    //   ...permission,
                    //   user_target,
                    //   permission: {
                    //     level: permission.level ?? Types.PermissionLevel.None,
                    //     specific,
                    //   },
                    // });
                  }}
                />
              );
            },
          },
        ]}
      />
    </Section>
  );
};
