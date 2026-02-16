import Tags from "@/components/tags";
import TagSelector from "@/components/tags/selector";
import { fmtDateWithMinutes } from "@/lib/formatting";
import { useInvalidate, useRead, useSetTitle, useWrite } from "@/lib/hooks";
import ResourceSelector from "@/resources/selector";
import { ICONS } from "@/theme/icons";
import { DataTable, SortableHeader } from "@/ui/data-table";
import { Badge, Group, Switch, TextInput } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { ColumnDef } from "@tanstack/react-table";
import { Types } from "komodo_client";
import { useMemo } from "react";
import DeleteOnboardingKey from "./delete";
import Section from "@/ui/section";
import NewOnboardingKey from "./new";

export default function SettingsOnboardingKeys() {
  useSetTitle("Onboarding");
  const { data } = useRead("ListOnboardingKeys", {});
  const keys = data ?? [];
  const invalidate = useInvalidate();
  const { mutate } = useWrite("UpdateOnboardingKey", {
    onSuccess: () => {
      invalidate(["ListOnboardingKeys"]);
      notifications.show({ message: "Updated onboarding key", color: "green" });
    },
  });
  const columns: (
    | ColumnDef<Types.OnboardingKey, unknown>
    | false
    | undefined
  )[] = useMemo(
    () => [
      {
        size: 150,
        accessorKey: "name",
        header: ({ column }) => <SortableHeader column={column} title="Name" />,
        cell: ({ row }) => (
          <TextInput
            defaultValue={row.original.name}
            onBlur={(e) =>
              e.target.value != row.original.name &&
              mutate({
                public_key: row.original.public_key,
                name: e.target.value,
              })
            }
            onKeyDown={(e) => e.key === "Enter" && e.currentTarget.blur()}
          />
        ),
      },
      {
        size: 150,
        accessorKey: "copy_server",
        header: "Template",
        cell: ({ row }) => (
          <ResourceSelector
            type="Server"
            selected={row.original.copy_server}
            templates={Types.TemplatesQueryBehavior.Include}
            onSelect={(copy_server) =>
              mutate({ public_key: row.original.public_key, copy_server })
            }
          />
        ),
      },
      {
        size: 200,
        accessorKey: "tags",
        header: "Tags",
        cell: ({ row }) => {
          const tags = useRead("ListTags", {}).data;
          const otherTags = tags?.filter(
            (tag) => !row.original.tags?.includes(tag._id?.$oid!),
          );
          return (
            <Group>
              <TagSelector
                title="Select Tags"
                tags={otherTags}
                onSelect={(tag) =>
                  mutate({
                    public_key: row.original.public_key,
                    tags: [...(row.original.tags ?? []), tag],
                  })
                }
                position="bottom-start"
              />

              <Tags
                tagIds={
                  tags
                    ?.filter((tag) => row.original.tags?.includes(tag.name))
                    .map((tag) => tag.name) ?? []
                }
                onBadgeClick={(toRemove) =>
                  mutate({
                    public_key: row.original.public_key,
                    tags: row.original.tags?.filter(
                      (tagName) => tagName !== toRemove,
                    ),
                  })
                }
                icon={<ICONS.Remove size="1rem" />}
                fz="1rem"
                useName
              />
            </Group>
          );
        },
      },
      {
        size: 100,
        accessorKey: "fix_existing_servers",
        header: ({ column }) => (
          <SortableHeader
            column={column}
            title="Priviledged"
            description="Allow the onboarding key to update an existing Server's public key and configuration to enable the connection."
          />
        ),
        cell: ({ row }) => (
          <Switch
            checked={row.original.fix_existing_servers}
            onChange={(e) =>
              mutate({
                public_key: row.original.public_key,
                fix_existing_servers: e.target.checked,
              })
            }
          />
        ),
      },
      {
        size: 100,
        accessorKey: "create_builder",
        header: ({ column }) => (
          <SortableHeader column={column} title="Create Builder" />
        ),
        cell: ({ row }) => (
          <Switch
            checked={row.original.create_builder}
            onChange={(e) =>
              mutate({
                public_key: row.original.public_key,
                create_builder: e.target.checked,
              })
            }
          />
        ),
      },
      {
        size: 100,
        accessorKey: "enabled",
        header: ({ column }) => (
          <SortableHeader column={column} title="Enabled" />
        ),
        cell: ({
          row: {
            original: { public_key, expires, enabled },
          },
        }) => (
          <Switch
            checked={expires && expires <= Date.now() ? false : enabled}
            onChange={(e) => mutate({ public_key, enabled: e.target.checked })}
            disabled={!!expires && expires <= Date.now()}
          />
        ),
      },
      {
        size: 150,
        accessorKey: "expires",
        header: ({ column }) => (
          <SortableHeader column={column} title="Expires" />
        ),
        cell: ({
          row: {
            original: { expires },
          },
        }) => (
          <Badge
            color={expires && expires <= Date.now() ? "red" : "green"}
            fz={{ base: "sm", lg: "md" }}
            p={{ base: "sm", lg: "md" }}
            size="md"
          >
            {expires ? fmtDateWithMinutes(new Date(expires)) : "Never"}
          </Badge>
        ),
      },
      {
        size: 100,
        accessorKey: "public_key",
        header: "Delete",
        cell: ({ row }) => (
          <DeleteOnboardingKey publicKey={row.original.public_key} />
        ),
      },
    ],
    [mutate],
  );

  return (
    <Section
      title="Server Onboarding Keys"
      icon={<ICONS.OnboardingKey className="w-5 h-5" />}
      actions={<NewOnboardingKey />}
    >
      <DataTable
        tableKey="server-onboarding-keys-v1"
        data={keys}
        columns={columns}
      />
    </Section>
  );
}
