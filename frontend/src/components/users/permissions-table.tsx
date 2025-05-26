import { useInvalidate, useWrite } from "@lib/hooks";
import { Types } from "komodo_client";
import { UsableResource } from "@types";
import { useToast } from "@ui/use-toast";
import { useState } from "react";
import { useUserTargetPermissions } from "./hooks";
import { Section } from "@components/layouts";
import { Input } from "@ui/input";
import { ResourceComponents } from "@components/resources";
import { Label } from "@ui/label";
import { Switch } from "@ui/switch";
import { DataTable, SortableHeader } from "@ui/data-table";
import { level_to_number, resource_name } from "@lib/utils";
import { ResourceLink } from "@components/resources/common";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { PermissionLevelSelector } from "@components/config/util";

export const PermissionsTable = ({
  user_target,
}: {
  user_target: Types.UserTarget;
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
      title="Specific Permissions"
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
                      specific: []
                    },
                  })
                }
              />
            ),
          },
        ]}
      />
    </Section>
  );
};
