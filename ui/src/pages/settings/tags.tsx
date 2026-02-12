import NewTag from "@/components/tags/new";
import UserAvatar from "@/components/user-avatar";
import { useRead, useSetTitle, useUser } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import { DataTable } from "@/ui/data-table";
import { Group, Stack, TextInput } from "@mantine/core";
import { useState } from "react";

export default function SettingsTags() {
  useSetTitle("Tags");
  const user = useUser().data!;

  const [search, setSearch] = useState("");

  const tags = useRead("ListTags", {}).data;

  return (
    <Stack>
      <Group justify="space-between">
        <NewTag />
        <TextInput
          placeholder="search..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          w={{ base: 200, lg: 300 }}
          leftSection={<ICONS.Search size="1rem" />}
        />
      </Group>

      <DataTable
        tableKey="tags"
        data={tags?.filter((tag) => tag.name.includes(search)) ?? []}
        columns={[
          {
            header: "Name",
            size: 200,
            accessorKey: "name",
          },
          {
            header: "Color",
            size: 200,
            cell: ({ row }) => (
              <></>
              // <ColorSelector
              //   tag_id={row.original._id?.$oid!}
              //   color={row.original.color!}
              //   disabled={!user.admin && row.original.owner !== user._id?.$oid}
              // />
            ),
          },
          {
            header: "Owner",
            size: 200,
            cell: ({ row }) =>
              row.original.owner ? (
                <UserAvatar userId={row.original.owner} />
              ) : (
                "Unknown"
              ),
          },
          {
            header: "Delete",
            size: 200,
            cell: ({ row }) => (
              <></>
              // <DeleteTag
              //   tag_id={row.original._id!.$oid}
              //   disabled={!user.admin && row.original.owner !== user._id?.$oid}
              // />
            ),
          },
        ]}
      />
    </Stack>
  );
}
