import ExportToml from "@/components/export-toml";
import { useRead } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import { ICONS } from "@/theme/icons";
import { DataTable } from "@/ui/data-table";
import Section from "@/ui/section";
import { Group } from "@mantine/core";
import { useNavigate } from "react-router-dom";
import DeleteUserGroup from "../../../components/user/delete-group";
import NewUserGroup from "./new-group";
import SearchInput from "@/ui/search-input";

export default function SettingsUserGroups({
  search,
  setSearch,
}: {
  search: string;
  setSearch: (search: string) => void;
}) {
  const nav = useNavigate();
  const groups = useRead("ListUserGroups", {}).data;
  const filtered = filterBySplit(groups, search, (group) => group.name);
  return (
    <Section title="User Groups" icon={<ICONS.Users size="1.3rem" />}>
      <Group justify="space-between">
        <NewUserGroup />
        <Group>
          <SearchInput value={search} onSearch={setSearch} />
          <ExportToml userGroups={groups?.map((g) => g._id?.$oid!)} />
        </Group>
      </Group>
      <DataTable
        tableKey="user-groups"
        data={filtered}
        columns={[
          { header: "Name", accessorKey: "name" },
          {
            header: "Members",
            accessorFn: (group) =>
              group.everyone ? "Everyone" : (group.users ?? []).length,
          },
          {
            header: "Delete",
            cell: ({ row: { original: group } }) => (
              <DeleteUserGroup group={group} />
            ),
          },
        ]}
        onRowClick={(group) => nav(`/user-groups/${group._id!.$oid}`)}
      />
    </Section>
  );
}
