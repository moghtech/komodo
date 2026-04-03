import { ICONS } from "@/lib/icons";
import { DividedChildren } from "mogh_ui";
import EntityHeader from "@/ui/entity-header";
import { Box, Group, Stack, Text } from "@mantine/core";
import { Types } from "komodo_client";
import UserAvatar from "@/components/user-avatar";
import UpdatesSection from "@/components/updates/section";

export interface UserHeaderProps {
  user: Types.User | undefined;
}

export default function UserHeader({ user }: UserHeaderProps) {
  const enabledIntent = user?.enabled ? "Good" : "Critical";
  const Header = (
    <Stack justify="space-between">
      <Stack gap="md" pb="md" className="bordered-light" bdrs="md">
        <EntityHeader
          name={user?.username}
          icon={ICONS.User}
          intent={enabledIntent}
          state={user?.enabled ? "Enabled" : "Disabled"}
        />
        <DividedChildren px="md">
          <Box>
            <UserAvatar userId={user?._id?.$oid!} onlyAvatar />
          </Box>
          <Text>
            Level:{" "}
            <b>
              {user?.super_admin
                ? "Super Admin"
                : user?.admin
                  ? "Admin"
                  : "User"}
            </b>
          </Text>
          <Text>
            Type: <b>{user?.config.type}</b>
          </Text>
        </DividedChildren>
      </Stack>
    </Stack>
  );

  return (
    <>
      <Stack hiddenFrom="lg" w="100%">
        {Header}
        <UpdatesSection
          query={user?._id?.$oid && { operator: user._id.$oid }}
        />
      </Stack>
      <Group
        visibleFrom="lg"
        gap="xl"
        w="100%"
        align="stretch"
        grow
        preventGrowOverflow={false}
      >
        {Header}
        <UpdatesSection
          query={user?._id?.$oid && { operator: user._id.$oid }}
        />
      </Group>
    </>
  );
}
