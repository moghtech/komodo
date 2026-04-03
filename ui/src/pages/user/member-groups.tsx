import { useInvalidate, useRead, useWrite } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import Section from "@/ui/section";
import { Box, Group, Text } from "@mantine/core";
import { useNavigate } from "react-router-dom";
import UserAddUserGroup from "./add-group";
import { notifications } from "@mantine/notifications";
import ConfirmIcon from "@/ui/confirm-icon";

export default function UserMemberGroups({ userId }: { userId: string }) {
  const allGroups = useRead("ListUserGroups", {}).data;
  const groups = allGroups?.filter((group) => group.users?.includes(userId));
  const nav = useNavigate();
  const inv = useInvalidate();
  const { mutate: remove, isPending: removePending } = useWrite(
    "RemoveUserFromUserGroup",
    {
      onSuccess: () => {
        inv(["ListUserGroups"]);
        notifications.show({ message: "Removed user from group" });
      },
    },
  );
  if (!allGroups || allGroups.length === 0) {
    return null;
  }
  return (
    <Section
      title="Groups"
      icon={<ICONS.UserGroup size="1.2rem" />}
      titleFz="h3"
     
      titleRight={
        <Box ml="md">
          <UserAddUserGroup userId={userId} />
        </Box>
      }
      withBorder
    >
      {groups?.length ? (
        <Group>
          {groups?.map((group) => (
            <Group
              key={group._id?.$oid}
              title={`User Group - ${group.name}`}
              onClick={() => nav(`/user-groups/${group._id?.$oid}`)}
              className="accent-hover-light bordered-heavy"
              py="xs"
              px="md"
              bdrs="sm"
              wrap="nowrap"
              gap="sm"
              style={{ cursor: "pointer" }}
            >
              <ICONS.UserGroup size="1rem" />
              <Text className="hover-underline" style={{ textWrap: "nowrap" }}>
                {group.name}
              </Text>
              <ConfirmIcon
                variant="filled"
                color="red"
                onClick={() =>
                  remove({ user_group: group._id?.$oid!, user: userId })
                }
                loading={removePending}
              >
                <ICONS.Remove size="1rem" />
              </ConfirmIcon>
            </Group>
          ))}
        </Group>
      ) : null}
    </Section>
  );
}
