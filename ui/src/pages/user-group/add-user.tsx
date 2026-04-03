import UserAvatar from "@/components/user-avatar";
import { useInvalidate, useRead, useWrite } from "@/lib/hooks";
import { filterBySplit } from "@/lib/utils";
import { ICONS } from "@/lib/icons";
import { Button, Combobox, ComboboxProps } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useSearchCombobox } from "mogh_ui";

export interface UserGroupAddUserProps extends ComboboxProps {
  groupId: string;
}

export default function UserGroupAddUser({
  groupId,
  position = "bottom-start",
  ...comboboxProps
}: UserGroupAddUserProps) {
  const { search, setSearch, combobox } = useSearchCombobox();

  const group = useRead("ListUserGroups", {}).data?.find(
    (group) => group._id?.$oid === groupId,
  );

  const users = useRead("ListUsers", {}).data?.filter(
    (user) =>
      // Only show users not already in group
      !group?.users?.includes(user._id?.$oid!),
  );

  const inv = useInvalidate();
  const { mutate: addUser, isPending } = useWrite("AddUserToUserGroup", {
    onSuccess: () => {
      inv(["ListUserGroups"]);
      notifications.show({
        message: "Added User to User Group",
        color: "green",
      });
    },
  });

  if (!users || users.length === 0) return null;

  const filtered = filterBySplit(users, search, (item) => item.username);

  return (
    <Combobox
      store={combobox}
      width={300}
      position={position}
      onOptionSubmit={(user) => {
        combobox.closeDropdown();
        addUser({ user_group: groupId, user });
      }}
      {...comboboxProps}
    >
      <Combobox.Target>
        <Button
          onClick={() => combobox.toggleDropdown()}
          leftSection={<ICONS.Add size="1rem" />}
          loading={isPending}
        >
          Add User
        </Button>
      </Combobox.Target>

      <Combobox.Dropdown>
        <Combobox.Search
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          leftSection={<ICONS.Search size="1rem" style={{ marginRight: 6 }} />}
          placeholder="search..."
        />
        <Combobox.Options mah={224} style={{ overflowY: "auto" }}>
          {filtered.map((user) => (
            <Combobox.Option key={user._id?.$oid!} value={user._id?.$oid!}>
              <UserAvatar userId={user._id?.$oid!} fz="md" />
            </Combobox.Option>
          ))}
          {filtered.length === 0 && (
            <Combobox.Empty>No results.</Combobox.Empty>
          )}
        </Combobox.Options>
      </Combobox.Dropdown>
    </Combobox>
  );
}
