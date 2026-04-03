import NewTag from "@/components/tags/new";
import Tag from "@/components/tags/tag";
import UserAvatar from "@/components/user-avatar";
import { useRead, useSetTitle, useUser } from "@/lib/hooks";
import { DataTable } from "mogh_ui";
import { Group, Stack } from "@mantine/core";
import { useState } from "react";
import TagColorSelector from "./color-selector";
import DeleteTag from "./delete";
import SearchInput from "@/ui/search-input";

export default function SettingsTags() {
  useSetTitle("Tags");
  const user = useUser().data!;

  const [search, setSearch] = useState("");

  const tags = useRead("ListTags", {}).data;

  return (
    <Stack>
      <Group justify="space-between">
        <NewTag />
        <SearchInput value={search} onSearch={setSearch} />
      </Group>

      <DataTable
        tableKey="tags"
        data={tags?.filter((tag) => tag.name.includes(search)) ?? []}
        columns={[
          {
            header: "Name",
            size: 200,
            accessorKey: "name",
            cell: ({ row }) => {
              return <Tag tag={row.original} size="md" fz="md" />;
            },
          },
          {
            header: "Color",
            size: 200,
            cell: ({ row }) => (
              <TagColorSelector
                tagId={row.original._id?.$oid!}
                color={row.original.color!}
                disabled={!user.admin && row.original.owner !== user._id?.$oid}
              />
            ),
          },
          {
            header: "Owner",
            size: 200,
            cell: ({ row }) =>
              row.original.owner ? (
                <UserAvatar userId={row.original.owner} fz="md" />
              ) : (
                "Unknown"
              ),
          },
          {
            header: "Delete",
            size: 200,
            cell: ({ row }) => (
              <DeleteTag
                tagId={row.original._id!.$oid}
                disabled={!user.admin && row.original.owner !== user._id?.$oid}
              />
            ),
          },
        ]}
      />
    </Stack>
  );
}
