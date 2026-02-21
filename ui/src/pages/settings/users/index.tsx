import { Group, Stack } from "@mantine/core";
import SettingsUserGroups from "./groups";
import {
  useInvalidate,
  useLoginOptions,
  useRead,
  useSetTitle,
  useUser,
  useWrite,
} from "@/lib/hooks";
import { useState } from "react";
import Section from "@/ui/section";
import { notifications } from "@mantine/notifications";
import { filterBySplit } from "@/lib/utils";
import { ICONS } from "@/theme/icons";
import UserTable from "@/components/user/table";
import { useNavigate } from "react-router-dom";
import NewServiceUser from "./new-service-user";
import NewLocalUser from "./new-local-user";

export default function SettingsUsers() {
  useSetTitle("Users");
  const inv = useInvalidate();
  const nav = useNavigate();
  const [search, setSearch] = useState("");
  const user = useUser().data;
  const localLoginEnabled = useLoginOptions().data?.local;
  const { mutateAsync: deleteUser } = useWrite("DeleteUser", {
    onSuccess: (user) => {
      notifications.show({
        message: `Deleted User ${user.username}.`,
        color: "green",
      });
      inv(["ListUsers"]);
    },
  });
  const users = useRead("ListUsers", {}).data;
  const filtered = filterBySplit(users, search, (user) => user.username);
  return (
    <Stack gap="xl">
      <SettingsUserGroups search={search} setSearch={setSearch} />

      <Section title="Users" icon={<ICONS.User size="1.3rem" />}>
        <Group>
          {localLoginEnabled && <NewLocalUser />}
          <NewServiceUser />
        </Group>
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
          onSelfClick={() => nav("/profile")}
        />
      </Section>
    </Stack>
  );
}
