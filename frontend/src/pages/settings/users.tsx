import { ExportButton } from "@components/export";
import { Section } from "@components/layouts";
import { DeleteUserGroup } from "@components/users/delete-user-group";
import {
  NewLocalUser,
  NewServiceUser,
  NewUserGroup,
} from "@components/users/new";
import { UserTable } from "@components/users/table";
import {
  useInvalidate,
  useLoginOptions,
  useRead,
  useSetTitle,
  useUser,
  useWrite,
} from "@lib/hooks";
import { filterBySplit } from "@lib/utils";
import { DataTable } from "@ui/data-table";
import { Input } from "@ui/input";
import { useToast } from "@ui/use-toast";
import { Search, User, Users } from "lucide-react";
import React, { useState } from "react";
import { useNavigate } from "react-router-dom";

export const UsersPage = ({ goToProfile }: { goToProfile: () => void }) => {
  useSetTitle("Users");
  const [search, setSearch] = useState("");
  return (
    <div className="flex flex-col gap-6">
      <UserGroupsSection search={search} setSearch={setSearch} />
      <UsersSection search={search} goToProfile={goToProfile} />
    </div>
  );
};

const UserGroupsSection = ({
  search,
  setSearch,
}: {
  search: string;
  setSearch: React.Dispatch<React.SetStateAction<string>>;
}) => {
  const nav = useNavigate();
  const groups = useRead("ListUserGroups", {}).data;
  const filtered = filterBySplit(groups, search, (group) => group.name);
  return (
    <Section title="User Groups" icon={<Users className="w-4 h-4" />}>
      <div className="flex items-center justify-between">
        <NewUserGroup />
        <div className="flex items-center gap-4">
          {groups && groups.length > 0 && (
            <div className="flex items-center gap-4">
              <ExportButton
                user_groups={groups
                  ?.map((group) => group._id?.$oid!)
                  .filter((id) => id)}
              />
            </div>
          )}
          <div className="relative">
            <Search className="w-4 absolute top-[50%] left-3 -translate-y-[50%] text-muted-foreground" />
            <Input
              placeholder="search..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="pl-8 w-[200px] lg:w-[300px]"
            />
          </div>
        </div>
      </div>
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
};

const UsersSection = ({
  goToProfile,
  search,
}: {
  goToProfile: () => void;
  search: string;
}) => {
  const user = useUser().data;
  const inv = useInvalidate();
  const { toast } = useToast();
  const local_login_enabled = useLoginOptions().data?.local;
  const { mutate: deleteUser } = useWrite("DeleteUser", {
    onSuccess: () => {
      toast({ title: "User deleted." });
      inv(["ListUsers"]);
    },
  });
  const users = useRead("ListUsers", {}).data;
  const filtered = filterBySplit(users, search, (user) => user.username);
  return (
    <Section title="Users" icon={<User className="w-4 h-4" />}>
      <div className="flex items-center gap-4">
        {local_login_enabled && <NewLocalUser />}
        <NewServiceUser />
      </div>
      <UserTable
        users={filtered}
        onUserDelete={
          user?.admin ? (user_id) => deleteUser({ user: user_id }) : undefined
        }
        userDeleteDisabled={(user_id) => {
          const toDelete = users?.find((user) => user._id?.$oid === user_id);
          if (!toDelete) return true;
          if (!toDelete.admin) return false;
          if (toDelete.super_admin) return true;
          return !user?.super_admin;
        }}
        onSelfClick={goToProfile}
      />
    </Section>
  );
};
