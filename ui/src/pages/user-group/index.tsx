import UserTable from "@/components/user/table";
import { useInvalidate, useRead, useUser, useWrite } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import DividedChildren from "@/ui/divided-children";
import EntityHeader from "@/ui/entity-header";
import EntityPage from "@/ui/entity-page";
import PageGuard from "@/ui/page-guard";
import Section from "@/ui/section";
import { Group, Stack, Text } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useParams } from "react-router-dom";
import UserGroupAddUser from "./add-user";
import LabelledSwitch from "@/ui/labelled-switch";
import BasePermissionsSection from "@/components/permissions/base-section";
import SpecificPermissionsSection from "@/components/permissions/specific-section";
import DeleteUserGroup from "./delete";

export default function UserGroup() {
  const adminUser = useUser().data;
  const inv = useInvalidate();
  const groupId = useParams().id as string;
  const group = useRead("ListUserGroups", {}).data?.find(
    (group) => group._id?.$oid === groupId,
  );

  const { data: users, isPending } = useRead("ListUsers", {});

  const { mutateAsync: rename, isPending: renamePending } = useWrite(
    "RenameUserGroup",
    {
      onSuccess: () => {
        inv(["ListUserGroups"]);
        notifications.show({ message: "Renamed User Group", color: "green" });
      },
    },
  );

  const removeMutate = useWrite("RemoveUserFromUserGroup", {
    onSuccess: () => {
      inv(["ListUserGroups"]);
      notifications.show({
        message: "Removed User from User Group",
        color: "green",
      });
    },
  }).mutate;

  const everyoneMutate = useWrite("SetEveryoneUserGroup", {
    onSuccess: () => {
      inv(["ListUserGroups"]);
      notifications.show({
        message: "Toggled User Group 'everyone'",
        color: "green",
      });
    },
  }).mutate;

  return (
    <PageGuard
      isPending={isPending}
      error={
        !adminUser?.admin
          ? "This page is only for admin users."
          : !group
            ? "Group could not be found"
            : undefined
      }
    >
      <EntityPage backTo="/settings">
        <Stack gap="md" pb="md" className="bordered-light" bdrs="md">
          <EntityHeader
            name={group?.name}
            icon={ICONS.UserGroup}
            intent="Good"
            onRename={(name) => rename({ id: groupId, name })}
            renamePending={renamePending}
            action={<DeleteUserGroup groupId={groupId} />}
          />
          <DividedChildren px="md">
            <Text>User Group</Text>
            <Text>
              {group?.everyone && "Everyone"}
              {!group?.everyone && (group?.users?.length ?? 0 > 0) && (
                <>
                  {(group?.users ?? []).length} User
                  {(group?.users ?? []).length > 1 ? "s" : ""}
                </>
              )}
              {!group?.everyone && (group?.users?.length ?? 0) === 0 && (
                <>No Users</>
              )}
            </Text>
          </DividedChildren>
        </Stack>

        <Stack mt="lg" gap="xl">
          <Section
            title="Users"
            titleFz="h3"
           
            icon={<ICONS.User size="1.2rem" />}
            titleRight={
              <Group ml="md">
                {!group?.everyone && <UserGroupAddUser groupId={groupId} />}
                <LabelledSwitch
                  label="Everyone"
                  checked={group?.everyone}
                  onCheckedChange={(everyone) =>
                    everyoneMutate({ user_group: groupId, everyone })
                  }
                />
                {group?.everyone && (
                  <Text c="dimmed">
                    All users will inherit the permissions in this group.
                  </Text>
                )}
              </Group>
            }
            withBorder
          >
            {!group?.everyone && (
              <UserTable
                noBorder
                users={
                  users?.filter((user) =>
                    group
                      ? (group.users ?? []).includes(user._id?.$oid!)
                      : false,
                  ) ?? []
                }
                onUserRemove={(user) =>
                  removeMutate({ user_group: groupId, user })
                }
              />
            )}
          </Section>

          <BasePermissionsSection
            userTarget={{ type: "UserGroup", id: groupId }}
            mt="md"
          />
          <SpecificPermissionsSection
            userTarget={{ type: "UserGroup", id: groupId }}
            mt="md"
          />
        </Stack>
      </EntityPage>
    </PageGuard>
  );
}
