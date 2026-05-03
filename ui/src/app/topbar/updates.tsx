import { useRead, useUser, useUserInvalidate, useWrite } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import {
  ActionIcon,
  Center,
  Menu,
  useComputedColorScheme,
} from "@mantine/core";
import { Circle } from "lucide-react";
import { useDisclosure } from "@mantine/hooks";
import { tagColor } from "@/lib/color";
import { Types } from "komodo_client";
import UpdateList from "@/components/updates/list";

export default function TopbarUpdates() {
  const [opened, { open, close }] = useDisclosure();
  const updates = useRead("ListUpdates", {}).data;

  const lastOpened = useUser().data?.last_update_view;
  const unseenUpdate = updates?.updates.some(
    (u) => u.start_ts > (lastOpened ?? Number.MAX_SAFE_INTEGER),
  );

  const userInvalidate = useUserInvalidate();
  const { mutate: setLastSeenUpdate } = useWrite("SetLastSeenUpdate", {
    onSuccess: userInvalidate,
  });

  const isDark = useComputedColorScheme() === "dark";
  const tagBlue = tagColor(
    isDark ? Types.TagColor.LightBlue : Types.TagColor.Blue,
  );

  return (
    <Menu
      opened={opened}
      position="bottom"
      offset={16}
      onOpen={() => {
        open();
        setLastSeenUpdate({});
      }}
      onClose={close}
    >
      <Menu.Target>
        <ActionIcon size="xl" variant="subtle" c="inherit">
          <Center pos="relative">
            <ICONS.Update size="1.3rem" />
            <Circle
              size="12px"
              style={{
                position: "absolute",
                top: "-6px",
                right: "-6px",
                opacity: unseenUpdate ? 1 : 0,
                transition: "all 300ms ease",
              }}
              color={tagBlue}
              fill={tagBlue}
            />
          </Center>
        </ActionIcon>
      </Menu.Target>
      <Menu.Dropdown>
        <UpdateList
          showAllLink="/updates"
          onUpdateClick={close}
          mah="min(500px, calc(100vh - 90px))"
          w={{ base: "92vw", md: 500, xl3: 600 }}
          large
        />
      </Menu.Dropdown>
    </Menu>
  );
}
