import { useRead } from "@/lib/hooks";
import { ICONS } from "@/theme/icons";
import Section from "@/ui/section";
import { Group, Text } from "@mantine/core";
import { Link } from "react-router-dom";

export default function UserMemberGroups({ userId }: { userId: string }) {
  const groups = useRead("ListUserGroups", {}).data?.filter((group) =>
    group.users?.includes(userId),
  );
  if (!groups || groups.length === 0) {
    return null;
  }
  return (
    <Section
      title="Groups"
      icon={<ICONS.UserGroup size="1.2rem" />}
      titleFz="h3"
      titleMb="0"
      withBorder
    >
      <Group>
        {groups.map((group) => (
          <Group
            key={group._id?.$oid}
            title={`User Group - ${group.name}`}
            renderRoot={(props) => (
              <Link to={`/user-groups/${group._id?.$oid}`} {...props} />
            )}
            className="accent-hover-light bordered-heavy"
            py="xs"
            px="md"
            bdrs="sm"
            wrap="nowrap"
            gap="xs"
          >
            <ICONS.UserGroup size="1rem" />
            <Text className="hover-underline" style={{ textWrap: "nowrap" }}>
              {group.name}
            </Text>
          </Group>
        ))}
      </Group>
    </Section>
  );
}
